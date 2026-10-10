mod members;
mod modules;
mod names;
mod references;

use std::collections::HashMap;
use std::ops::Range;

use crate::code::SyntaxNode;
use members::member_use;
use modules::ModuleItems;
use names::{Definable, class_name, definition_name};
use references::is_reference;

pub const INIT: &str = "init";

#[derive(Default)]
pub struct FileIndex {
    pub definitions: Vec<Definition>,
    pub bindings: HashMap<String, Binding>,
    pub exports: Vec<Export>,
    pub loads: Vec<Load>,
}

pub struct Definition {
    pub name: String,
    pub simple: String,
    pub range: Range<usize>,
    pub uses: Vec<Use>,
}

pub struct Use {
    pub line: usize,
    pub target: UseTarget,
}

pub enum UseTarget {
    Name(String),
    Member {
        object: String,
        property: String,
        call: bool,
    },
    Method(String),
    Load(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Load {
    pub specifier: String,
    pub line: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    pub specifier: String,
    pub imported: Imported,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Imported {
    Module,
    Named(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Export {
    Local {
        exported: String,
        local: String,
    },
    Forward {
        exported: Option<String>,
        specifier: String,
        imported: Option<String>,
    },
}

struct Visit<'r> {
    node: SyntaxNode<'r>,
    definition: usize,
    class: Option<String>,
}

pub fn index(root: &SyntaxNode) -> FileIndex {
    let mut index = FileIndex {
        definitions: vec![Definition {
            name: INIT.to_string(),
            simple: INIT.to_string(),
            range: root.range(),
            uses: Vec::new(),
        }],
        ..FileIndex::default()
    };
    let mut items = ModuleItems::default();
    let mut stack = vec![Visit {
        node: root.clone(),
        definition: 0,
        class: None,
    }];
    while let Some(visit) = stack.pop() {
        let node = &visit.node;
        let mut definition = visit.definition;
        let mut class = visit.class.clone();
        let mut skipped: Option<usize> = None;
        match node.kind().as_ref() {
            "function_declaration"
            | "generator_function_declaration"
            | "function_expression"
            | "function"
            | "generator_function"
            | "arrow_function"
            | "method_definition" => {
                if let Some((name, simple)) = definition_name(Definable {
                    node,
                    class: class.as_deref(),
                }) {
                    index.definitions.push(Definition {
                        name,
                        simple,
                        range: node.range(),
                        uses: Vec::new(),
                    });
                    definition = index.definitions.len() - 1;
                }
            }
            "class_declaration" | "class" | "abstract_class_declaration" => {
                class = Some(class_name(node));
            }
            "identifier" if is_reference(node) => {
                index.definitions[definition].uses.push(Use {
                    line: line_of(node),
                    target: UseTarget::Name(node.text().to_string()),
                });
            }
            "member_expression" => {
                if let Some(found) = member_use(node) {
                    skipped = found.skipped_child;
                    index.definitions[definition].uses.push(Use {
                        line: line_of(node),
                        target: found.target,
                    });
                }
            }
            "import_statement"
            | "export_statement"
            | "call_expression"
            | "assignment_expression"
            | "variable_declarator" => {
                let outcome = items.read(node);
                skipped = outcome.skipped_child;
                if let Some(target) = outcome.load {
                    index.definitions[definition].uses.push(Use {
                        line: line_of(node),
                        target,
                    });
                }
                if outcome.is_leaf {
                    continue;
                }
            }
            _ => {}
        }
        let children: Vec<SyntaxNode> = node.children().collect();
        for child in children.into_iter().rev() {
            if skipped == Some(child.node_id()) {
                continue;
            }
            stack.push(Visit {
                node: child,
                definition,
                class: class.clone(),
            });
        }
    }
    items.finish(&mut index);
    index
}

impl FileIndex {
    pub fn innermost(&self, offset: usize) -> usize {
        self.definitions
            .iter()
            .enumerate()
            .filter(|(_, definition)| definition.range.contains(&offset))
            .min_by_key(|(_, definition)| definition.range.len())
            .map_or(0, |(position, _)| position)
    }
}

pub fn line_of(node: &SyntaxNode) -> usize {
    node.start_pos().line() + 1
}
