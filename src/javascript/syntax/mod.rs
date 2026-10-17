mod members;
mod modules;
mod names;
mod references;

use crate::code::SyntaxNode;
pub use crate::deep::program::{
    Binding, Definition, Export, FileIndex, INIT, Imported, Load, Use, UseTarget,
};
use members::member_use;
use modules::{ModuleItems, constant_strings};
use names::{Definable, binds_name, class_name, definition_name};
use references::is_reference;

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
    let mut items = ModuleItems::with_constants(constant_strings(root));
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
                    if !binds_name(node) {
                        index.members.push(node.range());
                    }
                    definition = index.definitions.len() - 1;
                }
            }
            "class_declaration" | "class" | "abstract_class_declaration" => {
                class = Some(class_name(node));
            }
            "identifier" if is_reference(node) => {
                index.definitions[definition].uses.push(Use {
                    line: line_of(node),
                    start: node.range().start,
                    target: UseTarget::Name(node.text().to_string()),
                });
            }
            "member_expression" => {
                if let Some(found) = member_use(node) {
                    skipped = found.skipped_child;
                    index.definitions[definition].uses.push(Use {
                        line: line_of(node),
                        start: node.range().start,
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
                        start: node.range().start,
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

pub fn line_of(node: &SyntaxNode) -> usize {
    node.start_pos().line() + 1
}
