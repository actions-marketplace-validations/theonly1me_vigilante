mod bindings;
mod calls;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

pub use bindings::{CANDIDATE_MARK, NAMESPACE_SEPARATOR};
pub use calls::loaded;

use bindings::{ConstantInput, bind_constants};
use calls::{CONSTRUCTOR, CallSite, OwnCall, call_target, own_call, variable_types};

use crate::code::SyntaxNode;
use crate::deep::program::{Definition, Export, FileIndex, INIT, Load, Use, UseTarget};

const STATEMENT_HOLDERS: &[&str] = &[
    "body_statement",
    "block_body",
    "then",
    "else",
    "do",
    "begin",
];
const INITIALIZER: &str = "initialize";
const VISIBILITY_KEYWORDS: &[&str] = &["private", "protected", "public", "module_function"];

struct Visit<'r> {
    node: SyntaxNode<'r>,
    definition: usize,
    class: Option<String>,
    nesting: Vec<String>,
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
    let variables = variable_types(root);
    let mut constants: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut local = Vec::new();
    let mut stack = vec![Visit {
        node: root.clone(),
        definition: 0,
        class: None,
        nesting: Vec::new(),
    }];
    while let Some(visit) = stack.pop() {
        let node = &visit.node;
        let mut definition = visit.definition;
        let mut class = visit.class.clone();
        let mut nesting = visit.nesting.clone();
        let line = node.start_pos().line() + 1;
        let start = node.range().start;
        match node.kind().as_ref() {
            "module" | "class" => {
                if let Some(name) = node.field("name") {
                    let parts: Vec<String> = name
                        .text()
                        .split(NAMESPACE_SEPARATOR)
                        .filter(|part| !part.is_empty())
                        .map(str::to_string)
                        .collect();
                    if let Some(last) = parts.last() {
                        local.push(last.clone());
                        index.exports.push(Export::Local {
                            exported: last.clone(),
                            local: last.clone(),
                        });
                        class = Some(last.clone());
                    }
                    nesting.extend(parts);
                }
            }
            "method" | "singleton_method" => {
                let simple = node
                    .field("name")
                    .map(|name| name.text().to_string())
                    .unwrap_or_default();
                let simple = if simple == INITIALIZER {
                    CONSTRUCTOR.to_string()
                } else {
                    simple
                };
                index.definitions.push(Definition {
                    name: class
                        .as_ref()
                        .map_or_else(|| simple.clone(), |class| format!("{class}.{simple}")),
                    simple,
                    range: node.range(),
                    uses: Vec::new(),
                });
                definition = index.definitions.len() - 1;
            }
            "call" => {
                if let Some(specifier) = loaded(node) {
                    index.loads.push(Load {
                        specifier: specifier.clone(),
                        line,
                    });
                    index.definitions[definition].uses.push(Use {
                        line,
                        start,
                        target: UseTarget::Load(specifier),
                    });
                } else if let Some(target) = call_target(CallSite {
                    node,
                    class: class.as_deref(),
                    variables: &variables,
                }) {
                    if let UseTarget::Member { object, .. } = &target
                        && object.starts_with(char::is_uppercase)
                    {
                        constants.insert(object.clone(), nesting.clone());
                    }
                    index.definitions[definition].uses.push(Use {
                        line,
                        start,
                        target,
                    });
                }
            }
            "identifier"
                if node
                    .parent()
                    .is_some_and(|parent| STATEMENT_HOLDERS.contains(&parent.kind().as_ref()))
                    && !VISIBILITY_KEYWORDS.contains(&node.text().as_ref()) =>
            {
                index.definitions[definition].uses.push(Use {
                    line,
                    start,
                    target: own_call(OwnCall {
                        class: class.as_deref(),
                        method: node.text().to_string(),
                    }),
                });
            }
            _ => {}
        }
        let children: Vec<SyntaxNode> = node.children().collect();
        for child in children.into_iter().rev() {
            stack.push(Visit {
                node: child,
                definition,
                class: class.clone(),
                nesting: nesting.clone(),
            });
        }
    }
    bind_constants(ConstantInput {
        index: &mut index,
        constants,
        local: &local,
    });
    index
}
