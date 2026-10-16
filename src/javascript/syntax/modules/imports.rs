use super::nodes::{NodeChildren, literal, plain, unwrapped};
use super::{LoadSite, ModuleItems, NewBinding, Outcome};
use crate::code::SyntaxNode;
use crate::javascript::syntax::Imported;

struct ImportItem<'a, 'r> {
    item: &'a SyntaxNode<'r>,
    specifier: &'a str,
}

impl ModuleItems {
    pub(super) fn import(&mut self, node: &SyntaxNode) -> Outcome {
        let leaf = Outcome {
            is_leaf: true,
            ..Outcome::default()
        };
        if node.has_child("type") {
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
        self.load(LoadSite {
            specifier: &specifier,
            node,
        });
        for part in node.children().chain(clause) {
            for item in part.children() {
                self.import_item(ImportItem {
                    item: &item,
                    specifier: &specifier,
                });
            }
        }
        leaf
    }

    fn import_item(&mut self, import: ImportItem) {
        let ImportItem { item, specifier } = import;
        match item.kind().as_ref() {
            "identifier" => self.bind(NewBinding {
                local: &item.text(),
                specifier,
                imported: Imported::Module,
            }),
            "namespace_import" => {
                if let Some(name) = item.children().find(|child| child.kind() == "identifier") {
                    self.bind(NewBinding {
                        local: &name.text(),
                        specifier,
                        imported: Imported::Module,
                    });
                }
            }
            "named_imports" => {
                for named in item
                    .children()
                    .filter(|child| child.kind() == "import_specifier" && !child.has_child("type"))
                {
                    let Some(name) = named.field("name") else {
                        continue;
                    };
                    let local = named.field("alias").unwrap_or_else(|| name.clone());
                    self.bind(NewBinding {
                        local: &local.text(),
                        specifier,
                        imported: Imported::Named(plain(&name)),
                    });
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
        if let Some(specifier) = self.loaded_specifier(&value) {
            match name.kind().as_ref() {
                "identifier" => self.bind(NewBinding {
                    local: &name.text(),
                    specifier: &specifier,
                    imported: Imported::Module,
                }),
                "object_pattern" => {
                    for part in name.children() {
                        match part.kind().as_ref() {
                            "shorthand_property_identifier_pattern" => {
                                let local = part.text().to_string();
                                self.bind(NewBinding {
                                    local: &local,
                                    specifier: &specifier,
                                    imported: Imported::Named(local.clone()),
                                });
                            }
                            "pair_pattern" => {
                                if let (Some(key), Some(value)) =
                                    (part.field("key"), part.field("value"))
                                {
                                    self.bind(NewBinding {
                                        local: &value.text(),
                                        specifier: &specifier,
                                        imported: Imported::Named(plain(&key)),
                                    });
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
            && let Some(specifier) = self.loaded_specifier(&unwrapped(object))
        {
            self.bind(NewBinding {
                local: &name.text(),
                specifier: &specifier,
                imported: Imported::Named(plain(&property)),
            });
        }
    }
}
