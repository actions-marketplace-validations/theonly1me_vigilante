use std::collections::HashMap;

use crate::code::SyntaxNode;
use crate::deep::program::CanonicalCall;

const ALIASED_VALUES: &[&str] = &[
    "method_invocation",
    "object_creation_expression",
    "field_access",
];

pub fn canonical_calls<'r>(root: &SyntaxNode<'r>) -> Vec<CanonicalCall<'r>> {
    let aliases = aliases_of(root);
    if aliases.is_empty() {
        return Vec::new();
    }
    root.dfs()
        .filter(|node| node.kind() == "method_invocation")
        .filter_map(|call| {
            let object = call.field("object")?;
            if object.kind() != "identifier" {
                return None;
            }
            let replacement = aliases.get(object.text().as_ref())?;
            let text = call.text();
            let snippet = format!(
                "class Vigilante {{ void vigilante() {{ {replacement}{}; }} }}\n",
                &text[object.text().len()..]
            );
            Some(CanonicalCall {
                node: call,
                snippet,
            })
        })
        .collect()
}

fn aliases_of(root: &SyntaxNode) -> HashMap<String, String> {
    root.dfs()
        .filter(|node| node.kind() == "variable_declarator")
        .filter_map(|declarator| {
            let name = declarator.field("name")?;
            let value = declarator.field("value")?;
            ALIASED_VALUES
                .contains(&value.kind().as_ref())
                .then(|| (name.text().to_string(), value.text().to_string()))
        })
        .collect()
}
