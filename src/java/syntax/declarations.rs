use std::collections::{BTreeSet, HashMap};

use crate::code::SyntaxNode;
use crate::deep::program::{Binding, FileIndex, Imported, Load};

const TYPE_HOLDERS: &[&str] = &[
    "field_declaration",
    "local_variable_declaration",
    "formal_parameter",
];
const CLASS_KINDS: &[&str] = &[
    "class_declaration",
    "interface_declaration",
    "enum_declaration",
    "record_declaration",
    "annotation_type_declaration",
];

#[derive(Default)]
pub struct Declarations {
    pub package: String,
    pub wildcards: Vec<String>,
    pub variable_types: HashMap<String, String>,
    pub classes: BTreeSet<String>,
}

pub struct DeclarationsIn<'a, 'r> {
    pub index: &'a mut FileIndex,
    pub root: &'a SyntaxNode<'r>,
}

pub fn declarations(input: DeclarationsIn) -> Declarations {
    let DeclarationsIn { index, root } = input;
    let mut found = Declarations::default();
    for node in root.dfs() {
        let kind = node.kind();
        match kind.as_ref() {
            "package_declaration" => {
                if let Some(name) = node
                    .children()
                    .find(|child| child.kind().contains("identifier"))
                {
                    found.package = name.text().to_string();
                }
            }
            "import_declaration" => {
                if let Some(import) = import_of(&node) {
                    index.loads.push(Load {
                        specifier: import.specifier.clone(),
                        line: node.start_pos().line() + 1,
                    });
                    match import.local {
                        Some((local, imported)) => {
                            index.bindings.insert(
                                local,
                                Binding {
                                    specifier: import.specifier,
                                    imported,
                                },
                            );
                        }
                        None => found.wildcards.push(import.specifier),
                    }
                }
            }
            kind if TYPE_HOLDERS.contains(&kind) => {
                let Some(type_name) = node.field("type").map(|kind| simple_type(&kind.text()))
                else {
                    continue;
                };
                let names = node
                    .field_children("declarator")
                    .filter_map(|declarator| declarator.field("name"))
                    .chain(node.field("name"));
                for name in names {
                    found
                        .variable_types
                        .insert(name.text().to_string(), type_name.clone());
                }
            }
            kind if CLASS_KINDS.contains(&kind) => {
                if let Some(name) = node.field("name") {
                    found.classes.insert(name.text().to_string());
                }
            }
            _ => {}
        }
    }
    found
}

pub struct Import {
    pub specifier: String,
    pub local: Option<(String, Imported)>,
}

pub fn import_of(node: &SyntaxNode) -> Option<Import> {
    let path = node
        .children()
        .find(|child| child.kind() == "scoped_identifier" || child.kind() == "identifier")?
        .text()
        .to_string();
    let is_static = node.children().any(|child| child.kind() == "static");
    let is_wildcard = node.children().any(|child| child.kind() == "asterisk");
    if is_wildcard {
        return Some(Import {
            specifier: format!("{path}.*"),
            local: None,
        });
    }
    let (owner, last) = path.rsplit_once('.').unwrap_or(("", path.as_str()));
    Some(if is_static {
        Import {
            specifier: owner.to_string(),
            local: Some((last.to_string(), Imported::Named(last.to_string()))),
        }
    } else {
        Import {
            specifier: path.clone(),
            local: Some((last.to_string(), Imported::Module)),
        }
    })
}

pub fn simple_type(text: &str) -> String {
    text.split('<')
        .next()
        .unwrap_or(text)
        .trim_end_matches("[]")
        .trim()
        .to_string()
}
