use super::line_of;
use crate::code::SyntaxNode;
use crate::deep::program::{Binding, Export, FileIndex, Imported, Load};

const FUTURE: &str = "__future__";

pub struct ImportStatement<'a, 'r> {
    pub index: &'a mut FileIndex,
    pub statement: &'a SyntaxNode<'r>,
}

#[derive(Default)]
struct Read {
    loads: Vec<String>,
    bindings: Vec<(String, Binding)>,
    exports: Vec<Export>,
}

pub fn read_import(import: ImportStatement) {
    let ImportStatement { index, statement } = import;
    let read = if statement.kind() == "import_statement" {
        plain_import(statement)
    } else {
        from_import(statement)
    };
    let line = line_of(statement);
    index.loads.extend(
        read.loads
            .into_iter()
            .map(|specifier| Load { specifier, line }),
    );
    index.bindings.extend(read.bindings);
    index.exports.extend(read.exports);
}

fn plain_import(statement: &SyntaxNode) -> Read {
    let mut read = Read::default();
    for name in statement.field_children("name") {
        match name.kind().as_ref() {
            "dotted_name" => {
                let module = name.text().to_string();
                let parts: Vec<&str> = module.split('.').collect();
                for length in 1..=parts.len() {
                    let prefix = parts[..length].join(".");
                    read.bindings.push((
                        prefix.clone(),
                        Binding {
                            specifier: prefix,
                            imported: Imported::Module,
                        },
                    ));
                }
                read.loads.push(module);
            }
            "aliased_import" => {
                let (Some(module), Some(alias)) = (name.field("name"), name.field("alias")) else {
                    continue;
                };
                read.bindings.push((
                    alias.text().to_string(),
                    Binding {
                        specifier: module.text().to_string(),
                        imported: Imported::Module,
                    },
                ));
                read.loads.push(module.text().to_string());
            }
            _ => {}
        }
    }
    read
}

fn from_import(statement: &SyntaxNode) -> Read {
    let mut read = Read::default();
    let Some(module) = statement
        .field("module_name")
        .map(|module| module.text().to_string())
    else {
        return read;
    };
    if module == FUTURE {
        return read;
    }
    read.loads.push(module.clone());
    if statement
        .children()
        .any(|child| child.kind() == "wildcard_import")
    {
        read.exports.push(Export::Forward {
            exported: None,
            specifier: module.clone(),
            imported: None,
        });
    }
    for name in statement.field_children("name") {
        let (imported, local) = match name.kind().as_ref() {
            "dotted_name" => (name.text().to_string(), name.text().to_string()),
            "aliased_import" => {
                let (Some(imported), Some(alias)) = (name.field("name"), name.field("alias"))
                else {
                    continue;
                };
                (imported.text().to_string(), alias.text().to_string())
            }
            _ => continue,
        };
        let separator = if module.ends_with('.') { "" } else { "." };
        read.loads.push(format!("{module}{separator}{imported}"));
        read.exports.push(Export::Forward {
            exported: Some(local.clone()),
            specifier: module.clone(),
            imported: Some(imported.clone()),
        });
        read.bindings.push((
            local,
            Binding {
                specifier: module.clone(),
                imported: Imported::Named(imported),
            },
        ));
    }
    read
}
