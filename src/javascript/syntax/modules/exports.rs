use super::nodes::{
    MODULE_EXPORTS, arguments, bound_by_declaration, declared_names, has_child, is_anonymous,
    is_forwarded, literal, loaded_specifier, plain, specifiers, unwrapped,
};
use super::{ModuleItems, Outcome};
use crate::code::SyntaxNode;
use crate::javascript::syntax::UseTarget;

impl ModuleItems {
    pub(super) fn export(&mut self, node: &SyntaxNode) -> Outcome {
        let is_default = has_child(node, "default");
        if let Some(specifier) = node.field("source").and_then(|source| literal(&source)) {
            self.load(&specifier, node);
            let mut forwarded = false;
            for child in node.children() {
                match child.kind().as_ref() {
                    "export_clause" => {
                        for (name, alias) in specifiers(&child) {
                            self.forward(&specifier, (Some(alias), Some(name)));
                            forwarded = true;
                        }
                    }
                    "namespace_export" => {
                        let name = child.children().find(|part| part.kind() == "identifier");
                        self.forward(&specifier, (name.map(|name| plain(&name)), None));
                        forwarded = true;
                    }
                    _ => {}
                }
            }
            if !forwarded {
                self.forward(&specifier, (None, None));
            }
            return Outcome {
                is_leaf: true,
                ..Outcome::default()
            };
        }
        if let Some(clause) = node
            .children()
            .find(|child| child.kind() == "export_clause")
        {
            for (name, alias) in specifiers(&clause) {
                self.local(&alias, &name);
            }
        }
        let declared = node.field("declaration").or_else(|| node.field("value"));
        for name in declared.iter().flat_map(declared_names) {
            let exported = if is_default { "default" } else { &name };
            self.local(exported, &name);
        }
        if is_default && declared.as_ref().is_some_and(is_anonymous) {
            self.local("default", "default");
        }
        Outcome::default()
    }

    pub(super) fn call(&mut self, node: &SyntaxNode) -> Outcome {
        let callee = node.field("function");
        let callee_text = callee.as_ref().map(|callee| callee.text().to_string());
        if let Some(specifier) = loaded_specifier(node) {
            self.load(&specifier, node);
            let is_bound = bound_by_declaration(node) || is_forwarded(node);
            return Outcome {
                load: (!is_bound).then_some(UseTarget::Load(specifier)),
                skipped_child: callee.map(|callee| callee.node_id()),
                is_leaf: false,
            };
        }
        if callee_text
            .as_deref()
            .is_some_and(|text| text.ends_with("__exportStar") || text == "__export")
        {
            let first = arguments(node).into_iter().next();
            if let Some(specifier) = first.as_ref().and_then(loaded_specifier) {
                self.forward(&specifier, (None, None));
            }
        }
        Outcome::default()
    }

    pub(super) fn assignment(&mut self, node: &SyntaxNode) {
        let (Some(left), Some(right)) = (node.field("left"), node.field("right")) else {
            return;
        };
        let left_text = left.text();
        let exported = if left_text == MODULE_EXPORTS {
            None
        } else if let Some(("exports" | MODULE_EXPORTS, property)) = left_text.rsplit_once('.') {
            Some(property.to_string())
        } else {
            return;
        };
        let right = unwrapped(right);
        if let Some(specifier) = loaded_specifier(&right) {
            match &exported {
                None => {
                    self.forward(&specifier, (None, None));
                    let default = Some("default".to_string());
                    self.forward(&specifier, (default.clone(), default));
                }
                Some(name) => self.forward(&specifier, (Some(name.clone()), None)),
            }
            return;
        }
        let exported_name = exported.clone().unwrap_or_else(|| "default".to_string());
        match right.kind().as_ref() {
            "identifier" => self.local(&exported_name, &right.text()),
            "function_expression" | "arrow_function" | "class" | "generator_function" => {
                let own = right.field("name").map(|name| plain(&name));
                let local = match &exported {
                    Some(name) => name.clone(),
                    None => own.unwrap_or_else(|| "default".to_string()),
                };
                self.local(&exported_name, &local);
            }
            "object" if exported.is_none() => self.object_exports(&right),
            _ => {}
        }
    }

    fn object_exports(&mut self, object: &SyntaxNode) {
        for member in object.children() {
            match member.kind().as_ref() {
                "shorthand_property_identifier" => {
                    let name = member.text().to_string();
                    self.local(&name, &name);
                }
                "method_definition" => {
                    if let Some(name) = member.field("name") {
                        self.local(&plain(&name), &plain(&name));
                    }
                }
                "pair" => {
                    let (Some(key), Some(value)) = (member.field("key"), member.field("value"))
                    else {
                        continue;
                    };
                    let key = plain(&key);
                    let local = if value.kind() == "identifier" {
                        value.text().to_string()
                    } else {
                        key.clone()
                    };
                    self.local(&key, &local);
                }
                _ => {}
            }
        }
    }
}
