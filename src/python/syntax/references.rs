use crate::code::SyntaxNode;

const DECLARING_PARENTS: &[&str] = &[
    "parameters",
    "lambda_parameters",
    "typed_parameter",
    "list_splat_pattern",
    "dictionary_splat_pattern",
    "global_statement",
    "nonlocal_statement",
    "aliased_import",
    "dotted_name",
    "pair",
    "as_pattern_target",
];

pub fn is_reference(node: &SyntaxNode) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    let kind = parent.kind();
    if DECLARING_PARENTS.contains(&kind.as_ref()) {
        return false;
    }
    let is_field = |field: &str| {
        parent
            .field(field)
            .is_some_and(|child| child.node_id() == node.node_id())
    };
    match kind.as_ref() {
        "function_definition" | "class_definition" | "keyword_argument" => !is_field("name"),
        "default_parameter" | "typed_default_parameter" => !is_field("name"),
        "attribute" => !is_field("attribute"),
        "assignment" | "augmented_assignment" => !is_field("left") && !is_stored(&parent),
        "for_statement" | "for_in_clause" => !is_field("left"),
        "pattern_list" | "tuple_pattern" => false,
        _ => true,
    }
}

pub fn is_called(node: &SyntaxNode) -> bool {
    node.parent().is_some_and(|parent| {
        parent.kind() == "call"
            && parent
                .field("function")
                .is_some_and(|callee| callee.node_id() == node.node_id())
    })
}

pub fn is_storage_target(node: &SyntaxNode) -> bool {
    node.parent().is_some_and(|parent| {
        matches!(
            parent.kind().as_ref(),
            "assignment" | "augmented_assignment"
        ) && parent
            .field("left")
            .is_some_and(|left| left.node_id() == node.node_id())
    })
}

fn is_stored(assignment: &SyntaxNode) -> bool {
    assignment
        .field("left")
        .is_some_and(|left| left.kind() == "attribute" || left.kind() == "subscript")
}
