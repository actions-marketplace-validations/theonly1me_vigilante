use std::collections::HashMap;

use crate::code::SyntaxNode;
use crate::deep::program::CanonicalCall;

pub fn canonical_calls<'r>(root: &SyntaxNode<'r>) -> Vec<CanonicalCall<'r>> {
    let aliases = aliases_of(root);
    if aliases.is_empty() {
        return Vec::new();
    }
    root.dfs()
        .filter(|node| node.kind() == "call")
        .filter_map(|call| {
            let function = call.field("function")?;
            let text = function.text();
            let head = text.split('.').next().unwrap_or_default();
            let replacement = aliases.get(head)?;
            let arguments = call.field("arguments")?.text().to_string();
            let snippet = format!("{replacement}{}{arguments}\n", &text[head.len()..]);
            Some(CanonicalCall {
                node: call,
                snippet,
            })
        })
        .collect()
}

fn aliases_of(root: &SyntaxNode) -> HashMap<String, String> {
    let mut aliases = HashMap::new();
    for node in root.dfs() {
        match node.kind().as_ref() {
            "import_from_statement" => {
                let Some(module) = node.field("module_name") else {
                    continue;
                };
                let module = module.text().to_string();
                if module.starts_with('.') {
                    continue;
                }
                let names = node
                    .children()
                    .skip_while(|child| child.kind() != "import")
                    .skip(1);
                for name in names {
                    let (imported, local) = match name.kind().as_ref() {
                        "dotted_name" => (name.text().to_string(), name.text().to_string()),
                        "aliased_import" => {
                            let (Some(imported), Some(local)) =
                                (name.field("name"), name.field("alias"))
                            else {
                                continue;
                            };
                            (imported.text().to_string(), local.text().to_string())
                        }
                        _ => continue,
                    };
                    aliases.insert(local, format!("{module}.{imported}"));
                }
            }
            "import_statement" => {
                for name in node
                    .children()
                    .filter(|child| child.kind() == "aliased_import")
                {
                    if let (Some(imported), Some(local)) = (name.field("name"), name.field("alias"))
                    {
                        aliases.insert(local.text().to_string(), imported.text().to_string());
                    }
                }
            }
            _ => {}
        }
    }
    aliases
}
