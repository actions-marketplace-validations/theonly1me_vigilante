mod bindings;
mod calls;
mod declarations;
#[cfg(test)]
mod tests;

pub use bindings::CANDIDATE_MARK;

use bindings::{BindingInput, bind_types};
use calls::{CONSTRUCTOR, CallSite, target_of};
use declarations::declarations;

use crate::code::SyntaxNode;
use crate::deep::program::{Definition, Export, FileIndex, INIT, Use, UseTarget};

const TYPE_KINDS: &[&str] = &[
    "class_declaration",
    "struct_declaration",
    "interface_declaration",
    "record_declaration",
    "enum_declaration",
];

struct Visit<'r> {
    node: SyntaxNode<'r>,
    definition: usize,
    class: Option<String>,
    namespace: String,
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
    let found = declarations(root);
    let mut namespaces = Vec::new();
    let mut stack = vec![Visit {
        node: root.clone(),
        definition: 0,
        class: None,
        namespace: String::new(),
    }];
    while let Some(visit) = stack.pop() {
        let node = &visit.node;
        let mut definition = visit.definition;
        let mut class = visit.class.clone();
        let mut namespace = visit.namespace.clone();
        let kind = node.kind();
        let name = || node.field("name").map(|name| name.text().to_string());
        let mut defined = None;
        match kind.as_ref() {
            "namespace_declaration" | "file_scoped_namespace_declaration" => {
                namespace = name().unwrap_or_default();
                namespaces.push(namespace.clone());
            }
            kind if TYPE_KINDS.contains(&kind) => {
                class = name();
                if let Some(name) = &class {
                    index.exports.push(Export::Local {
                        exported: name.clone(),
                        local: name.clone(),
                    });
                }
            }
            "method_declaration" | "local_function_statement" => defined = name(),
            "constructor_declaration" => {
                let is_static = node
                    .children()
                    .any(|child| child.kind() == "modifier" && child.text() == "static");
                if !is_static {
                    defined = Some(CONSTRUCTOR.to_string());
                }
            }
            "accessor_declaration" => {
                let accessor = node.text();
                let property = node
                    .ancestors()
                    .find(|ancestor| ancestor.kind() == "property_declaration")
                    .and_then(|property| property.field("name"))
                    .map(|name| name.text().to_string());
                let has_body = node.children().any(|child| {
                    matches!(child.kind().as_ref(), "block" | "arrow_expression_clause")
                });
                if let (Some(property), true) = (property, has_body) {
                    let prefix = if accessor.trim_start().starts_with("get") {
                        "get"
                    } else {
                        "set"
                    };
                    defined = Some(format!("{prefix}_{property}"));
                }
            }
            "invocation_expression" | "object_creation_expression" => {
                if let Some(target) = target_of(CallSite {
                    node,
                    class: class.as_deref(),
                    declarations: &found,
                }) {
                    index.definitions[definition].uses.push(Use {
                        line: node.start_pos().line() + 1,
                        target,
                    });
                }
            }
            _ => {}
        }
        if let Some(simple) = defined {
            index.definitions.push(Definition {
                name: class
                    .as_ref()
                    .map_or_else(|| simple.clone(), |class| format!("{class}.{simple}")),
                simple,
                range: node.range(),
                uses: vec![Use {
                    line: node.start_pos().line() + 1,
                    target: UseTarget::Name(INIT.to_string()),
                }],
            });
            definition = index.definitions.len() - 1;
        }
        let children: Vec<SyntaxNode> = node.children().collect();
        for child in children.into_iter().rev() {
            stack.push(Visit {
                node: child,
                definition,
                class: class.clone(),
                namespace: namespace.clone(),
            });
        }
    }
    index.supertypes.clone_from(&found.supertypes);
    bind_types(BindingInput {
        index: &mut index,
        declarations: &found,
        namespaces: &namespaces,
    });
    index
}
