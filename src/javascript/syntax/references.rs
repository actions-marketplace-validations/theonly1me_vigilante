use crate::code::SyntaxNode;

const DECLARING_PARENTS: &[&str] = &[
    "import_specifier",
    "import_clause",
    "namespace_import",
    "import_require_clause",
    "export_specifier",
    "namespace_export",
    "required_parameter",
    "optional_parameter",
    "formal_parameters",
    "rest_pattern",
    "array_pattern",
    "pair_pattern",
    "catch_clause",
    "labeled_statement",
    "break_statement",
    "continue_statement",
    "pair",
    "export_statement",
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
        "variable_declarator" => !is_field("name"),
        "function_declaration"
        | "function_expression"
        | "class_declaration"
        | "class"
        | "generator_function_declaration" => !is_field("name"),
        "arrow_function" => !is_field("parameter"),
        "assignment_expression" | "augmented_assignment_expression" => {
            !is_field("left") && !is_stored(&parent)
        }
        "for_in_statement" => !is_field("left"),
        "assignment_pattern" => !is_field("left"),
        _ => true,
    }
}

pub fn is_assignment_target(node: &SyntaxNode) -> bool {
    node.parent().is_some_and(|parent| {
        parent.kind() == "assignment_expression"
            && parent
                .field("left")
                .is_some_and(|left| left.node_id() == node.node_id())
    })
}

pub fn is_called(node: &SyntaxNode) -> bool {
    node.parent().is_some_and(|parent| {
        let field = match parent.kind().as_ref() {
            "call_expression" => "function",
            "new_expression" => "constructor",
            _ => return false,
        };
        parent
            .field(field)
            .is_some_and(|callee| callee.node_id() == node.node_id())
    })
}

fn is_stored(assignment: &SyntaxNode) -> bool {
    assignment
        .field("left")
        .is_some_and(|left| left.kind() == "member_expression")
}
