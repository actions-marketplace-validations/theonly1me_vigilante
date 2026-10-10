mod calls;
mod paths;
mod test_items;
#[cfg(test)]
mod tests;
mod uses;

use std::collections::HashSet;

pub use uses::leaves;

use crate::code::SyntaxNode;
use crate::deep::program::{Binding, Definition, Export, FileIndex, INIT, Imported, Load, Use};
use calls::{Pending, PendingCall, pending_call};
use paths::{PathUse, path_target};
use test_items::is_test_item;
use uses::{UseDeclaration, read_use, segments};

const EXPORTED_ITEMS: &[&str] = &[
    "function_item",
    "struct_item",
    "enum_item",
    "union_item",
    "trait_item",
    "type_item",
    "const_item",
    "static_item",
];

struct Visit<'r> {
    node: SyntaxNode<'r>,
    definition: usize,
    owner: Option<String>,
    impl_type: Option<String>,
}

struct Found {
    definition: usize,
    line: usize,
    pending: Pending,
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
    let inline_modules: HashSet<String> = root
        .dfs()
        .filter(|node| node.kind() == "mod_item" && node.field("body").is_some())
        .filter_map(|node| node.field("name").map(|name| name.text().to_string()))
        .collect();
    let mut found = Vec::new();
    let mut stack = vec![Visit {
        node: root.clone(),
        definition: 0,
        owner: None,
        impl_type: None,
    }];
    while let Some(visit) = stack.pop() {
        let node = &visit.node;
        if is_test_item(node) {
            index.ignored.push(node.range());
            continue;
        }
        let mut definition = visit.definition;
        let mut owner = visit.owner.clone();
        let mut impl_type = visit.impl_type.clone();
        let kind = node.kind();
        let line = node.start_pos().line() + 1;
        let name = || node.field("name").map(|name| name.text().to_string());
        if EXPORTED_ITEMS.contains(&kind.as_ref())
            && visit.definition == 0
            && visit.owner.is_none()
            && let Some(name) = name()
        {
            index.exports.push(Export::Local {
                exported: name.clone(),
                local: name,
            });
        }
        match kind.as_ref() {
            "use_declaration" => {
                read_use(UseDeclaration {
                    index: &mut index,
                    declaration: node,
                    inline_modules: &inline_modules,
                });
                continue;
            }
            "extern_crate_declaration" => {
                let names: Vec<String> = node
                    .children()
                    .filter(|child| child.kind() == "identifier")
                    .map(|child| child.text().to_string())
                    .collect();
                if let Some(crate_name) = names.first() {
                    index.bindings.insert(
                        names.last().cloned().unwrap_or_default(),
                        Binding {
                            specifier: crate_name.clone(),
                            imported: Imported::Module,
                        },
                    );
                    index.loads.push(Load {
                        specifier: crate_name.clone(),
                        line,
                    });
                }
                continue;
            }
            "mod_item" => {
                if let (Some(module), None) = (name(), node.field("body")) {
                    index.loads.push(Load {
                        specifier: format!("self::{module}"),
                        line,
                    });
                }
            }
            "impl_item" => {
                impl_type = node
                    .field("type")
                    .and_then(|kind| segments(&kind.text()).pop());
                owner.clone_from(&impl_type);
            }
            "trait_item" => {
                impl_type = name();
                owner.clone_from(&impl_type);
            }
            "function_signature_item" | "macro_invocation" | "macro_definition" => continue,
            "function_item" => {
                let simple = name().unwrap_or_default();
                index.definitions.push(Definition {
                    name: owner
                        .as_ref()
                        .map_or_else(|| simple.clone(), |owner| format!("{owner}.{simple}")),
                    simple,
                    range: node.range(),
                    uses: Vec::new(),
                });
                definition = index.definitions.len() - 1;
                owner = None;
            }
            "call_expression" => {
                if let Some(pending) = node.field("function").and_then(|function| {
                    pending_call(PendingCall {
                        function: &function,
                        impl_type: impl_type.as_deref(),
                    })
                }) {
                    found.push(Found {
                        definition,
                        line,
                        pending,
                    });
                }
            }
            _ => {}
        }
        let children: Vec<SyntaxNode> = node.children().collect();
        for child in children.into_iter().rev() {
            stack.push(Visit {
                node: child,
                definition,
                owner: owner.clone(),
                impl_type: impl_type.clone(),
            });
        }
    }
    for item in found {
        let target = match item.pending {
            Pending::Ready(target) => Some(target),
            Pending::Path { text, impl_type } => path_target(PathUse {
                index: &mut index,
                text: &text,
                impl_type: impl_type.as_deref(),
                inline_modules: &inline_modules,
                line: item.line,
            }),
        };
        if let Some(target) = target {
            index.definitions[item.definition].uses.push(Use {
                line: item.line,
                target,
            });
        }
    }
    index
}
