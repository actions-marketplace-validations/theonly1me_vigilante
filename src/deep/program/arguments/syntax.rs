use crate::code::SyntaxNode;

const CALL_KINDS: &[&str] = &[
    "call_expression",
    "call",
    "new_expression",
    "method_invocation",
    "object_creation_expression",
    "invocation_expression",
];
const LITERAL_KINDS: &[&str] = &[
    "string",
    "string_literal",
    "raw_string_literal",
    "interpreted_string_literal",
    "template_string",
    "character_literal",
    "integer",
    "float",
    "number",
    "integer_literal",
    "float_literal",
    "decimal_integer_literal",
    "boolean_literal",
    "true",
    "false",
    "null",
    "null_literal",
    "none",
    "nil",
    "simple_symbol",
];
const CONTAINER_KINDS: &[&str] = &[
    "list",
    "array",
    "array_expression",
    "tuple",
    "argument_list",
    "arguments",
    "parenthesized_expression",
    "concatenated_string",
];
const INTERPOLATION_KINDS: &[&str] = &[
    "interpolation",
    "template_substitution",
    "interpolated_string_expression",
];
const OPTION_KINDS: &[&str] = &[
    "object",
    "hash",
    "arrow_function",
    "function_expression",
    "function",
    "lambda",
    "block",
    "do_block",
    "comment",
];
const COMMAND_KEYWORDS: &[&str] = &["args", "command", "cmd"];
const ASSIGNMENT_KINDS: &[(&str, &str, &str)] = &[
    ("assignment", "left", "right"),
    ("assignment_expression", "left", "right"),
    ("variable_declarator", "name", "value"),
    ("let_declaration", "pattern", "value"),
];

pub fn is_call(node: &SyntaxNode) -> bool {
    CALL_KINDS.contains(&node.kind().as_ref())
}

pub fn callee_text(call: &SyntaxNode) -> String {
    ["function", "constructor", "name", "method", "type"]
        .iter()
        .find_map(|field| call.field(field))
        .map(|callee| callee.text().to_string())
        .unwrap_or_default()
}

pub fn is_constant_command(arguments: &SyntaxNode) -> bool {
    arguments
        .children()
        .filter(|argument| argument.is_named() && !is_option(argument))
        .all(|argument| is_literal(&argument))
}

fn is_option(argument: &SyntaxNode) -> bool {
    let kind = argument.kind();
    if OPTION_KINDS.contains(&kind.as_ref()) {
        return true;
    }
    let name = match kind.as_ref() {
        "keyword_argument" | "named_argument" => argument.field("name"),
        "pair" => argument.field("key"),
        _ => return false,
    };
    name.is_none_or(|name| !COMMAND_KEYWORDS.contains(&name.text().trim_start_matches(':')))
}

pub fn is_literal(node: &SyntaxNode) -> bool {
    if node
        .dfs()
        .any(|inner| INTERPOLATION_KINDS.contains(&inner.kind().as_ref()))
    {
        return false;
    }
    let kind = node.kind();
    if LITERAL_KINDS.contains(&kind.as_ref()) {
        return true;
    }
    if CONTAINER_KINDS.contains(&kind.as_ref()) {
        return node
            .children()
            .filter(|child| child.is_named())
            .all(|child| is_literal(&child));
    }
    match kind.as_ref() {
        "keyword_argument" | "named_argument" | "pair" => {
            node.field("value").is_some_and(|value| is_literal(&value))
        }
        "binary_expression" | "binary_operator" => {
            let sides = (node.field("left"), node.field("right"));
            matches!(sides, (Some(left), Some(right)) if is_literal(&left) && is_literal(&right))
        }
        _ => false,
    }
}

pub fn assignments<'r>(scope: &SyntaxNode<'r>) -> Vec<(String, String, usize)> {
    scope
        .dfs()
        .filter_map(|node| {
            let (_, name_field, value_field) = ASSIGNMENT_KINDS
                .iter()
                .find(|(kind, _, _)| node.kind() == *kind)?;
            let name = node.field(name_field)?;
            let value = node.field(value_field)?;
            Some((
                name.text().to_string(),
                value.text().to_string(),
                node.range().start,
            ))
        })
        .collect()
}

pub fn parameter_names(function: &SyntaxNode) -> Vec<String> {
    let Some(parameters) = function
        .field("parameters")
        .or_else(|| function.field("parameter"))
    else {
        return Vec::new();
    };
    parameters
        .dfs()
        .filter(|node| node.kind() == "identifier")
        .map(|node| node.text().to_string())
        .collect()
}

pub fn identifiers(text: &str) -> impl Iterator<Item = &str> {
    text.split(|character: char| !character.is_alphanumeric() && character != '_')
        .filter(|word| {
            word.chars()
                .next()
                .is_some_and(|first| first.is_alphabetic() || first == '_')
        })
}

pub fn contains_source(text: &str, sources: &[&str]) -> bool {
    sources.iter().any(|source| {
        text.match_indices(source).any(|(start, _)| {
            let is_word = |character: char| character.is_alphanumeric() || character == '_';
            let open_before = text[..start]
                .chars()
                .next_back()
                .is_none_or(|before| !is_word(before));
            let open_after = !source.ends_with(is_word)
                || text[start + source.len()..]
                    .chars()
                    .next()
                    .is_none_or(|after| !is_word(after));
            open_before && open_after
        })
    })
}
