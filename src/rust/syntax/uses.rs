use std::collections::HashSet;

use crate::code::SyntaxNode;
use crate::deep::program::{Binding, Export, FileIndex, Imported, Load};

pub const SEPARATOR: &str = "::";
const SELF: &str = "self";

#[derive(Debug)]
pub struct Leaf {
    pub path: Vec<String>,
    pub alias: Option<String>,
    pub is_glob: bool,
}

impl Leaf {
    pub fn specifier(&self) -> String {
        if self.is_glob || self.path.len() == 1 {
            self.path.join(SEPARATOR)
        } else {
            self.path[..self.path.len() - 1].join(SEPARATOR)
        }
    }
}

pub fn leaves(declaration: &SyntaxNode) -> Vec<Leaf> {
    let mut found = Vec::new();
    if let Some(argument) = declaration.field("argument") {
        expand(Expansion {
            node: &argument,
            prefix: &[],
            found: &mut found,
        });
    }
    found
}

struct Expansion<'a, 'r> {
    node: &'a SyntaxNode<'r>,
    prefix: &'a [String],
    found: &'a mut Vec<Leaf>,
}

fn expand(expansion: Expansion) {
    let Expansion {
        node,
        prefix,
        found,
    } = expansion;
    let joined =
        |text: &str| -> Vec<String> { prefix.iter().cloned().chain(segments(text)).collect() };
    match node.kind().as_ref() {
        "use_as_clause" => found.push(Leaf {
            path: joined(
                &node
                    .field("path")
                    .map(|path| path.text().to_string())
                    .unwrap_or_default(),
            ),
            alias: node.field("alias").map(|alias| alias.text().to_string()),
            is_glob: false,
        }),
        "use_wildcard" => found.push(Leaf {
            path: joined(
                node.text()
                    .trim_end_matches('*')
                    .trim_end_matches(SEPARATOR),
            ),
            alias: None,
            is_glob: true,
        }),
        "scoped_use_list" => {
            let base = joined(
                &node
                    .field("path")
                    .map(|path| path.text().to_string())
                    .unwrap_or_default(),
            );
            if let Some(list) = node.field("list") {
                for child in list.children().filter(SyntaxNode::is_named) {
                    expand(Expansion {
                        node: &child,
                        prefix: &base,
                        found,
                    });
                }
            }
        }
        "use_list" => {
            for child in node.children().filter(SyntaxNode::is_named) {
                expand(Expansion {
                    node: &child,
                    prefix,
                    found,
                });
            }
        }
        _ => {
            let mut path = joined(&node.text());
            if path.last().is_some_and(|last| last == SELF) && path.len() > 1 {
                path.pop();
            }
            found.push(Leaf {
                path,
                alias: None,
                is_glob: false,
            });
        }
    }
}

pub fn segments(text: &str) -> Vec<String> {
    let mut depth = 0;
    let without_generics: String = text
        .chars()
        .filter(|character| {
            match *character {
                '<' => depth += 1,
                '>' => depth -= 1,
                _ => return depth == 0 && !character.is_whitespace(),
            }
            false
        })
        .collect();
    without_generics
        .split(SEPARATOR)
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect()
}

pub struct UseDeclaration<'a, 'r> {
    pub index: &'a mut FileIndex,
    pub declaration: &'a SyntaxNode<'r>,
    pub inline_modules: &'a HashSet<String>,
}

pub fn read_use(read: UseDeclaration) {
    let UseDeclaration {
        index,
        declaration,
        inline_modules,
    } = read;
    let line = declaration.start_pos().line() + 1;
    let is_public = declaration
        .children()
        .any(|child| child.kind() == "visibility_modifier");
    for leaf in leaves(declaration) {
        let specifier = leaf.specifier();
        let mut loads = if leaf.path.len() > 1 || leaf.is_glob {
            vec![specifier.clone()]
        } else {
            Vec::new()
        };
        if leaf.is_glob {
            if is_public {
                index.exports.push(Export::Forward {
                    exported: None,
                    specifier: specifier.clone(),
                    imported: None,
                });
            }
        } else if let Some(last) = leaf.path.last().cloned() {
            let local = leaf.alias.clone().unwrap_or_else(|| last.clone());
            let imported = if leaf.path.len() == 1 {
                Imported::Module
            } else {
                loads.push(leaf.path.join(SEPARATOR));
                if is_public {
                    index.exports.push(Export::Forward {
                        exported: Some(local.clone()),
                        specifier: specifier.clone(),
                        imported: Some(last.clone()),
                    });
                }
                Imported::Named(last)
            };
            index.bindings.insert(
                local,
                Binding {
                    specifier,
                    imported,
                },
            );
        }
        if leaf
            .path
            .first()
            .is_some_and(|first| inline_modules.contains(first))
        {
            continue;
        }
        index
            .loads
            .extend(loads.into_iter().map(|specifier| Load { specifier, line }));
    }
}
