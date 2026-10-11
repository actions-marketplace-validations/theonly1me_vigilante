mod declarations;
#[cfg(test)]
mod tests;
mod uses;

use declarations::{Declarations, DeclarationsIn, declarations};
use uses::{UseIn, use_of};

use crate::code::SyntaxNode;
use crate::deep::program::{
    Binding, Definition, Export, FileIndex, INIT, Imported, Load, Use, UseTarget,
};

pub use declarations::import_of;

const CONSTRUCTOR: &str = "constructor";
const CLASS_KINDS: &[&str] = &[
    "class_declaration",
    "interface_declaration",
    "enum_declaration",
    "record_declaration",
    "annotation_type_declaration",
];

struct Visit<'r> {
    node: SyntaxNode<'r>,
    definition: usize,
    class: String,
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
    let found = declarations(DeclarationsIn {
        index: &mut index,
        root,
    });
    let mut stack = vec![Visit {
        node: root.clone(),
        definition: 0,
        class: String::new(),
    }];
    while let Some(visit) = stack.pop() {
        let node = &visit.node;
        let mut definition = visit.definition;
        let mut class = visit.class.clone();
        let kind = node.kind();
        match kind.as_ref() {
            kind if CLASS_KINDS.contains(&kind) => {
                class = node
                    .field("name")
                    .map_or(class, |name| name.text().to_string());
            }
            "method_declaration" | "constructor_declaration" => {
                let simple = if kind == "constructor_declaration" {
                    CONSTRUCTOR.to_string()
                } else {
                    node.field("name")
                        .map(|name| name.text().to_string())
                        .unwrap_or_default()
                };
                let name = format!("{class}.{simple}");
                index.exports.push(Export::Local {
                    exported: simple.clone(),
                    local: name.clone(),
                });
                index.definitions.push(Definition {
                    name,
                    simple,
                    range: node.range(),
                    uses: vec![Use {
                        line: node.start_pos().line() + 1,
                        target: UseTarget::Name(INIT.to_string()),
                    }],
                });
                definition = index.definitions.len() - 1;
            }
            "import_declaration" | "package_declaration" => continue,
            _ => {
                if let Some(target) = use_of(UseIn {
                    node,
                    declarations: &found,
                    bindings: &index.bindings,
                }) {
                    index.definitions[definition].uses.push(Use {
                        line: node.start_pos().line() + 1,
                        target,
                    });
                }
            }
        }
        let children: Vec<SyntaxNode> = node.children().collect();
        for child in children.into_iter().rev() {
            stack.push(Visit {
                node: child,
                definition,
                class: class.clone(),
            });
        }
    }
    bind_types(BindTypes {
        index: &mut index,
        found: &found,
    });
    index
}

struct BindTypes<'a> {
    index: &'a mut FileIndex,
    found: &'a Declarations,
}

fn bind_types(input: BindTypes) {
    let BindTypes { index, found } = input;
    index.supertypes.clone_from(&found.supertypes);
    let objects: Vec<String> = index
        .definitions
        .iter()
        .flat_map(|definition| &definition.uses)
        .filter_map(|item| match &item.target {
            UseTarget::Member { object, .. } if object != "this" => Some(object.clone()),
            _ => None,
        })
        .chain(
            found
                .supertypes
                .iter()
                .flat_map(|supertypes| supertypes.names.iter().cloned()),
        )
        .collect();
    for object in objects {
        if index.bindings.contains_key(&object) || found.classes.contains(&object) {
            continue;
        }
        let specifier = if object.contains('.') {
            object.clone()
        } else if let Some(wildcard) = found.wildcards.first() {
            wildcard.clone()
        } else if found.package.is_empty() {
            object.clone()
        } else {
            format!("{}.{object}", found.package)
        };
        let imported = if specifier.ends_with(".*") {
            Imported::Named(object.clone())
        } else {
            Imported::Module
        };
        if !specifier.ends_with(".*") {
            index.loads.push(Load {
                specifier: specifier.clone(),
                line: 1,
            });
        }
        index.bindings.insert(
            object,
            Binding {
                specifier,
                imported,
            },
        );
    }
}
