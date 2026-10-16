use std::collections::HashMap;

use crate::code::SyntaxNode;
use crate::deep::program::CanonicalCall;

const PATH_SEPARATOR: &str = "::";

pub fn canonical_calls<'r>(root: &SyntaxNode<'r>) -> Vec<CanonicalCall<'r>> {
    let aliases = aliases_of(root);
    if aliases.is_empty() {
        return Vec::new();
    }
    root.dfs()
        .filter(|node| node.kind() == "call_expression")
        .filter_map(|call| {
            let function = call.field("function")?;
            if function.kind() != "scoped_identifier" {
                return None;
            }
            let text = function.text();
            let (head, rest) = text.split_once(PATH_SEPARATOR)?;
            let replacement = aliases.get(head)?;
            let arguments = call.field("arguments")?.text().to_string();
            let snippet =
                format!("fn vigilante() {{ {replacement}{PATH_SEPARATOR}{rest}{arguments}; }}\n");
            Some(CanonicalCall {
                node: call,
                snippet,
            })
        })
        .collect()
}

fn aliases_of(root: &SyntaxNode) -> HashMap<String, String> {
    root.dfs()
        .filter(|node| node.kind() == "use_as_clause")
        .filter_map(|clause| {
            let path = clause.field("path")?.text().to_string();
            let alias = clause.field("alias")?.text().to_string();
            let prefix = clause
                .parent()
                .filter(|parent| parent.kind() == "use_list")
                .and_then(|list| list.parent())
                .filter(|scoped| scoped.kind() == "scoped_use_list")
                .and_then(|scoped| scoped.field("path"))
                .map(|path| format!("{}{PATH_SEPARATOR}", path.text()))
                .unwrap_or_default();
            Some((alias, format!("{prefix}{path}")))
        })
        .collect()
}
