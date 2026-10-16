use std::collections::HashMap;

use crate::code::SyntaxNode;
use crate::deep::program::CanonicalCall;

const GLOBAL_OBJECTS: &[&str] = &["globalThis.", "window.", "self.", "global."];
const LOADERS: &[&str] = &["require", "import"];

#[derive(Default)]
struct Aliases {
    expressions: HashMap<String, String>,
    strings: HashMap<String, String>,
}

pub fn canonical_calls<'r>(root: &SyntaxNode<'r>) -> Vec<CanonicalCall<'r>> {
    let aliases = aliases_of(root);
    root.dfs()
        .filter(|node| matches!(node.kind().as_ref(), "call_expression" | "new_expression"))
        .filter_map(|call| {
            let is_new = call.kind() == "new_expression";
            let callee = call
                .field(if is_new { "constructor" } else { "function" })?
                .text()
                .to_string();
            let arguments = call
                .field("arguments")
                .map(|arguments| arguments.text().to_string())
                .unwrap_or_default();
            let snippet = if LOADERS.contains(&callee.as_str()) {
                let argument = arguments
                    .trim_start_matches('(')
                    .trim_end_matches(')')
                    .trim();
                format!("{callee}({});\n", aliases.strings.get(argument)?)
            } else {
                let canonical = canonical_callee(&callee, &aliases);
                if canonical == callee {
                    return None;
                }
                let new = if is_new { "new " } else { "" };
                format!("{new}{canonical}{arguments};\n")
            };
            Some(CanonicalCall {
                node: call,
                snippet,
            })
        })
        .collect()
}

fn canonical_callee(callee: &str, aliases: &Aliases) -> String {
    let head = callee.split('.').next().unwrap_or_default();
    let substituted = match aliases.expressions.get(head) {
        Some(replacement) => format!("{replacement}{}", &callee[head.len()..]),
        None => callee.to_string(),
    };
    GLOBAL_OBJECTS
        .iter()
        .find_map(|global| substituted.strip_prefix(global))
        .map_or(substituted.clone(), str::to_string)
}

fn aliases_of(root: &SyntaxNode) -> Aliases {
    let mut aliases = Aliases::default();
    for declarator in root
        .dfs()
        .filter(|node| node.kind() == "variable_declarator")
    {
        let (Some(name), Some(value)) = (declarator.field("name"), declarator.field("value"))
        else {
            continue;
        };
        if name.kind() != "identifier" {
            continue;
        }
        let name = name.text().to_string();
        match value.kind().as_ref() {
            "identifier" | "member_expression" => {
                aliases.expressions.insert(name, value.text().to_string());
            }
            "string" => {
                aliases.strings.insert(name, value.text().to_string());
            }
            _ => {}
        }
    }
    aliases
}
