use std::collections::{BTreeMap, HashMap, HashSet};

use crate::code::SyntaxNode;
use crate::deep::program::Supertypes;

const TYPE_KINDS: &[&str] = &[
    "class_declaration",
    "struct_declaration",
    "interface_declaration",
    "record_declaration",
    "enum_declaration",
];
const VARIABLE_HOLDERS: &[&str] = &[
    "field_declaration",
    "local_declaration_statement",
    "event_field_declaration",
];

#[derive(Default)]
pub struct Declarations {
    pub supertypes: Vec<Supertypes>,
    pub usings: Vec<String>,
    pub aliases: BTreeMap<String, String>,
    pub local_types: HashSet<String>,
    pub variable_types: HashMap<String, String>,
}

pub fn declarations(root: &SyntaxNode) -> Declarations {
    let mut found = Declarations::default();
    for node in root.dfs() {
        let kind = node.kind();
        match kind.as_ref() {
            "using_directive" => read_using(UsingRead {
                node: &node,
                found: &mut found,
            }),
            kind if TYPE_KINDS.contains(&kind) => {
                if let Some(name) = node.field("name") {
                    found.local_types.insert(name.text().to_string());
                    let names = base_names(&node);
                    if !names.is_empty() {
                        found.supertypes.push(Supertypes {
                            class: name.text().to_string(),
                            names,
                        });
                    }
                }
            }
            "parameter" => {
                if let (Some(kind), Some(name)) = (node.field("type"), node.field("name")) {
                    found
                        .variable_types
                        .insert(name.text().to_string(), type_name(&kind.text()));
                }
            }
            kind if VARIABLE_HOLDERS.contains(&kind) => {
                let Some(declaration) = node
                    .children()
                    .find(|child| child.kind() == "variable_declaration")
                else {
                    continue;
                };
                let declared = declaration
                    .field("type")
                    .map(|kind| kind.text().to_string());
                for declarator in declaration
                    .children()
                    .filter(|child| child.kind() == "variable_declarator")
                {
                    let Some(name) = declarator
                        .children()
                        .find(|child| child.kind() == "identifier")
                    else {
                        continue;
                    };
                    let inferred = declarator
                        .children()
                        .find(|child| child.kind() == "object_creation_expression")
                        .and_then(|creation| creation.field("type"))
                        .map(|kind| kind.text().to_string());
                    let chosen = match declared.as_deref() {
                        Some("var") | None => inferred,
                        Some(text) => Some(text.to_string()),
                    };
                    if let Some(chosen) = chosen {
                        found
                            .variable_types
                            .insert(name.text().to_string(), type_name(&chosen));
                    }
                }
            }
            _ => {}
        }
    }
    found
}

struct UsingRead<'a, 'r> {
    node: &'a SyntaxNode<'r>,
    found: &'a mut Declarations,
}

fn read_using(read: UsingRead) {
    let UsingRead { node, found } = read;
    let text = node.text();
    let body = text
        .trim()
        .trim_start_matches("global ")
        .trim_start_matches("using")
        .trim_end_matches(';')
        .trim();
    if body.starts_with("static ") {
        return;
    }
    match body.split_once('=') {
        Some((alias, target)) => {
            found
                .aliases
                .insert(alias.trim().to_string(), type_name(target.trim()));
        }
        None => found.usings.push(body.to_string()),
    }
}

pub fn type_name(text: &str) -> String {
    let mut depth = 0;
    text.chars()
        .filter(|character| {
            match *character {
                '<' => depth += 1,
                '>' => depth -= 1,
                _ => {
                    return depth == 0 && !character.is_whitespace() && !"?[]".contains(*character);
                }
            }
            false
        })
        .collect::<String>()
        .trim_start_matches("global::")
        .to_string()
}

fn base_names(declaration: &SyntaxNode) -> Vec<String> {
    declaration
        .children()
        .filter(|child| child.kind() == "base_list")
        .flat_map(|list| {
            list.children()
                .filter(SyntaxNode::is_named)
                .collect::<Vec<_>>()
        })
        .filter_map(|base| match base.kind().as_ref() {
            "identifier" | "qualified_name" => Some(base.text().to_string()),
            "generic_name" => base
                .children()
                .find(|child| child.kind() == "identifier")
                .map(|name| name.text().to_string()),
            _ => None,
        })
        .collect()
}
