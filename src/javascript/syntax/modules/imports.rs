use super::nodes::{has_child, literal, loaded_specifier, plain, unwrapped};
use super::{ModuleItems, Outcome};
use crate::code::SyntaxNode;
use crate::javascript::syntax::Imported;

impl ModuleItems {
    pub(super) fn import(&mut self, node: &SyntaxNode) -> Outcome {
        let leaf = Outcome {
            is_leaf: true,
            ..Outcome::default()
        };
        if has_child(node, "type") {
            return leaf;
        }
        let clause = node
            .children()
            .find(|child| child.kind() == "import_require_clause");
        let source = node
            .field("source")
            .or_else(|| clause.as_ref().and_then(|clause| clause.field("source")));
        let Some(specifier) = source.and_then(|source| literal(&source)) else {
            return leaf;
        };
        self.load(&specifier, node);
        for part in node.children().chain(clause) {
            for item in part.children() {
                self.import_item(&item, &specifier);
            }
        }
        leaf
    }

    fn import_item(&mut self, item: &SyntaxNode, specifier: &str) {
        match item.kind().as_ref() {
            "identifier" => self.bind(&item.text(), specifier, Imported::Module),
            "namespace_import" => {
                if let Some(name) = item.children().find(|child| child.kind() == "identifier") {
                    self.bind(&name.text(), specifier, Imported::Module);
                }
            }
            "named_imports" => {
                for named in item
                    .children()
                    .filter(|child| child.kind() == "import_specifier" && !has_child(child, "type"))
                {
                    let Some(name) = named.field("name") else {
                        continue;
                    };
                    let local = named.field("alias").unwrap_or_else(|| name.clone());
                    self.bind(&local.text(), specifier, Imported::Named(plain(&name)));
                }
            }
            _ => {}
        }
    }

    pub(super) fn declarator(&mut self, node: &SyntaxNode) {
        let (Some(name), Some(value)) = (node.field("name"), node.field("value")) else {
            return;
        };
        let value = unwrapped(value);
        if let Some(specifier) = loaded_specifier(&value) {
            match name.kind().as_ref() {
                "identifier" => self.bind(&name.text(), &specifier, Imported::Module),
                "object_pattern" => {
                    for part in name.children() {
                        match part.kind().as_ref() {
                            "shorthand_property_identifier_pattern" => {
                                let local = part.text().to_string();
                                self.bind(&local, &specifier, Imported::Named(local.clone()));
                            }
                            "pair_pattern" => {
                                if let (Some(key), Some(value)) =
                                    (part.field("key"), part.field("value"))
                                {
                                    self.bind(
                                        &value.text(),
                                        &specifier,
                                        Imported::Named(plain(&key)),
                                    );
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
            return;
        }
        if value.kind() == "member_expression"
            && name.kind() == "identifier"
            && let (Some(object), Some(property)) = (value.field("object"), value.field("property"))
            && let Some(specifier) = loaded_specifier(&unwrapped(object))
        {
            self.bind(&name.text(), &specifier, Imported::Named(plain(&property)));
        }
    }
}
