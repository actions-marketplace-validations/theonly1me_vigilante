use super::nodes::NodeChildren;
use super::nodes::{
    arguments, bound_by_declaration, declared_names, is_anonymous, is_forwarded, is_loader,
    literal, loaded_specifier, plain, specifiers,
};
use super::{LoadSite, ModuleItems, Outcome};
use crate::code::SyntaxNode;
use crate::javascript::syntax::{Export, UseTarget};

impl ModuleItems {
    pub(super) fn export(&mut self, node: &SyntaxNode) -> Outcome {
        let is_default = node.has_child("default");
        if let Some(specifier) = node.field("source").and_then(|source| literal(&source)) {
            self.load(LoadSite {
                specifier: &specifier,
                node,
            });
            let mut forwarded = false;
            for child in node.children() {
                match child.kind().as_ref() {
                    "export_clause" => {
                        for (name, alias) in specifiers(&child) {
                            self.add_export(Export::Forward {
                                exported: Some(alias),
                                specifier: specifier.to_string(),
                                imported: Some(name),
                            });
                            forwarded = true;
                        }
                    }
                    "namespace_export" => {
                        let name = child.children().find(|part| part.kind() == "identifier");
                        self.add_export(Export::Forward {
                            exported: name.map(|name| plain(&name)),
                            specifier: specifier.to_string(),
                            imported: None,
                        });
                        forwarded = true;
                    }
                    _ => {}
                }
            }
            if !forwarded {
                self.add_export(Export::Forward {
                    exported: None,
                    specifier: specifier.to_string(),
                    imported: None,
                });
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
                self.add_export(Export::Local {
                    exported: alias.to_string(),
                    local: name.to_string(),
                });
            }
        }
        let declared = node.field("declaration").or_else(|| node.field("value"));
        for name in declared.iter().flat_map(declared_names) {
            let exported = if is_default { "default" } else { &name };
            self.add_export(Export::Local {
                exported: exported.to_string(),
                local: name.to_string(),
            });
        }
        if is_default && declared.as_ref().is_some_and(is_anonymous) {
            self.add_export(Export::Local {
                exported: "default".to_string(),
                local: "default".to_string(),
            });
        }
        Outcome::default()
    }

    pub(super) fn call(&mut self, node: &SyntaxNode) -> Outcome {
        if is_loader(node) && loaded_specifier(node).is_none() {
            self.unresolved(node);
        }
        let callee = node.field("function");
        let callee_text = callee.as_ref().map(|callee| callee.text().to_string());
        if let Some(specifier) = loaded_specifier(node) {
            self.load(LoadSite {
                specifier: &specifier,
                node,
            });
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
                self.add_export(Export::Forward {
                    exported: None,
                    specifier: specifier.to_string(),
                    imported: None,
                });
            }
        }
        Outcome::default()
    }
}
