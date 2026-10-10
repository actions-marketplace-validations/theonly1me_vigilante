use super::ModuleItems;
use super::nodes::{MODULE_EXPORTS, loaded_specifier, plain, unwrapped};
use crate::code::SyntaxNode;
use crate::javascript::syntax::Export;

impl ModuleItems {
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
                    self.add_export(Export::Forward {
                        exported: None,
                        specifier: specifier.to_string(),
                        imported: None,
                    });
                    let default = Some("default".to_string());
                    self.add_export(Export::Forward {
                        exported: default.clone(),
                        specifier: specifier.to_string(),
                        imported: default,
                    });
                }
                Some(name) => self.add_export(Export::Forward {
                    exported: Some(name.clone()),
                    specifier: specifier.to_string(),
                    imported: None,
                }),
            }
            return;
        }
        let exported_name = exported.clone().unwrap_or_else(|| "default".to_string());
        match right.kind().as_ref() {
            "identifier" => self.add_export(Export::Local {
                exported: exported_name.to_string(),
                local: right.text().to_string(),
            }),
            "function_expression" | "arrow_function" | "class" | "generator_function" => {
                let own = right.field("name").map(|name| plain(&name));
                let local = match &exported {
                    Some(name) => name.clone(),
                    None => own.unwrap_or_else(|| "default".to_string()),
                };
                self.add_export(Export::Local {
                    exported: exported_name.to_string(),
                    local: local.to_string(),
                });
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
                    self.add_export(Export::Local {
                        exported: name.to_string(),
                        local: name.to_string(),
                    });
                }
                "method_definition" => {
                    if let Some(name) = member.field("name") {
                        self.add_export(Export::Local {
                            exported: plain(&name),
                            local: plain(&name),
                        });
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
                    self.add_export(Export::Local {
                        exported: key.to_string(),
                        local: local.to_string(),
                    });
                }
                _ => {}
            }
        }
    }
}
