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

pub fn definition_name(node: &SyntaxNode, class: Option<&str>) -> Option<(String, String)> {
    let own_name = node.field("name").map(|name| plain(&name.text()));
    if node.kind() == "method_definition" {
        let simple = own_name?;
        let owner = match class {
            Some(class) => Some(class.to_string()),
            None => node.parent().and_then(|object| holder_name(&object)),
        };
        return Some((qualified(owner.as_deref(), &simple), simple));
    }
    if node.kind() == "function_declaration" || node.kind() == "generator_function_declaration" {
        let simple = own_name?;
        return Some((simple.clone(), simple));
    }
    let parent = node.parent()?;
    let simple = match parent.kind().as_ref() {
        "variable_declarator" => name_of_identifier(&parent.field("name")?)?,
        "assignment_expression" => {
            let left = parent.field("left")?;
            match left.kind().as_ref() {
                "identifier" => left.text().to_string(),
                "member_expression" if left.text() == "module.exports" => {
                    own_name.unwrap_or_else(|| "default".to_string())
                }
                "member_expression" => {
                    let property = plain(&left.field("property")?.text());
                    let owner = left
                        .field("object")
                        .and_then(|object| owner_of(&object, class));
                    return Some((qualified(owner.as_deref(), &property), property));
                }
                _ => return None,
            }
        }
        "pair" => {
            let key = plain(&parent.field("key")?.text());
            let owner = parent.parent().and_then(|object| holder_name(&object));
            return Some((qualified(owner.as_deref(), &key), key));
        }
        "public_field_definition" => {
            let field = plain(&parent.field("name")?.text());
            return Some((qualified(class, &field), field));
        }
        "export_statement" => "default".to_string(),
        _ => own_name?,
    };
    Some((simple.clone(), simple))
}

pub fn class_name(node: &SyntaxNode) -> String {
    if let Some(name) = node.field("name") {
        return name.text().to_string();
    }
    node.parent()
        .filter(|parent| parent.kind() == "variable_declarator")
        .and_then(|parent| parent.field("name"))
        .map_or_else(|| "default".to_string(), |name| name.text().to_string())
}

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

fn holder_name(object: &SyntaxNode) -> Option<String> {
    let holder = object.parent()?;
    match holder.kind().as_ref() {
        "variable_declarator" => name_of_identifier(&holder.field("name")?),
        "assignment_expression" => {
            let left = holder.field("left")?;
            match left.text().as_ref() {
                "module.exports" | "exports" => None,
                _ => owner_of(&left, None),
            }
        }
        _ => None,
    }
}

fn owner_of(object: &SyntaxNode, class: Option<&str>) -> Option<String> {
    let text = object.text();
    match text.as_ref() {
        "exports" | "module.exports" => None,
        "this" => class.map(str::to_string),
        _ => Some(
            text.trim_end_matches(".prototype")
                .rsplit('.')
                .next()
                .unwrap_or_default()
                .to_string(),
        ),
    }
}

fn name_of_identifier(node: &SyntaxNode) -> Option<String> {
    (node.kind() == "identifier").then(|| node.text().to_string())
}

fn qualified(owner: Option<&str>, name: &str) -> String {
    match owner {
        Some(owner) if !owner.is_empty() => format!("{owner}.{name}"),
        _ => name.to_string(),
    }
}

fn plain(text: &str) -> String {
    text.trim_matches(['"', '\'', '`', '[', ']'])
        .trim_start_matches('#')
        .to_string()
}
