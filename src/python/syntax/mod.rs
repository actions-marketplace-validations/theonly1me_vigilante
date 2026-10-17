mod chains;
mod imports;
mod references;

use chains::{Chain, ChainUses, chain_of, resolve_chains};
use imports::{ImportStatement, read_import};
use references::is_reference;

use crate::code::SyntaxNode;
use crate::deep::program::{Definition, Export, FileIndex, INIT, Use, UseTarget};

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
    let mut chains: Vec<Chain> = Vec::new();
    let mut stack = vec![Visit {
        node: root.clone(),
        definition: 0,
        class: None,
    }];
    while let Some(visit) = stack.pop() {
        let node = &visit.node;
        let mut definition = visit.definition;
        let mut class = visit.class.clone();
        let mut skipped = None;
        let is_top_level = visit.definition == 0 && visit.class.is_none();
        match node.kind().as_ref() {
            "function_definition" => {
                let simple = node
                    .field("name")
                    .map(|name| name.text().to_string())
                    .unwrap_or_default();
                let name = class
                    .as_ref()
                    .map_or_else(|| simple.clone(), |class| format!("{class}.{simple}"));
                if is_top_level {
                    index.exports.push(Export::Local {
                        exported: simple.clone(),
                        local: simple.clone(),
                    });
                }
                if class.is_some() {
                    index.members.push(node.range());
                }
                index.definitions.push(Definition {
                    name,
                    simple,
                    range: node.range(),
                    uses: Vec::new(),
                });
                definition = index.definitions.len() - 1;
                class = None;
            }
            "class_definition" => {
                let name = node
                    .field("name")
                    .map(|name| name.text().to_string())
                    .unwrap_or_default();
                if is_top_level {
                    index.exports.push(Export::Local {
                        exported: name.clone(),
                        local: name.clone(),
                    });
                }
                class = Some(name);
            }
            "import_statement" | "import_from_statement" => {
                read_import(ImportStatement {
                    index: &mut index,
                    statement: node,
                });
                continue;
            }
            "attribute" => {
                if let Some(chain) = chain_of(node) {
                    skipped = chain.skipped_child;
                    chains.push(Chain {
                        definition,
                        line: line_of(node),
                        start: node.range().start,
                        ..chain
                    });
                } else if references::is_called(node)
                    && let Some(method) = node.field("attribute")
                {
                    index.definitions[definition].uses.push(Use {
                        line: line_of(node),
                        start: node.range().start,
                        target: UseTarget::Method(method.text().to_string()),
                    });
                }
            }
            "type" => continue,
            "identifier" if is_reference(node) => {
                index.definitions[definition].uses.push(Use {
                    line: line_of(node),
                    start: node.range().start,
                    target: UseTarget::Name(node.text().to_string()),
                });
            }
            _ => {}
        }
        let is_except_clause = node.kind() == "except_clause";
        let children: Vec<SyntaxNode> = node
            .children()
            .filter(|child| !is_except_clause || child.kind() == "block")
            .collect();
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
    resolve_chains(ChainUses {
        index: &mut index,
        chains,
    });
    index
}

pub fn line_of(node: &SyntaxNode) -> usize {
    node.start_pos().line() + 1
}
