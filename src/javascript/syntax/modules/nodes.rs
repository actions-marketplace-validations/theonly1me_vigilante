use crate::code::SyntaxNode;

pub const MODULE_EXPORTS: &str = "module.exports";
const WRAPPERS: &[&str] = &["await_expression", "parenthesized_expression"];

pub fn loaded_specifier(node: &SyntaxNode) -> Option<String> {
    if node.kind() != "call_expression" {
        return None;
    }
    let callee = node.field("function")?;
    let is_loader = match callee.kind().as_ref() {
        "identifier" => callee.text() == "require",
        "import" => true,
        _ => false,
    };
    if !is_loader {
        return None;
    }
    arguments(node).first().and_then(literal)
}

pub fn bound_by_declaration(call: &SyntaxNode) -> bool {
    let mut current = call.clone();
    while let Some(parent) = current.parent() {
        match parent.kind().as_ref() {
            kind if WRAPPERS.contains(&kind) => current = parent,
            "member_expression" => current = parent,
            "variable_declarator" => return true,
            _ => return false,
        }
    }
    false
}

pub fn is_forwarded(call: &SyntaxNode) -> bool {
    let Some(parent) = call.parent() else {
        return false;
    };
    match parent.kind().as_ref() {
        "assignment_expression" => parent.field("left").is_some_and(|left| {
            let text = left.text();
            text == MODULE_EXPORTS
                || text.starts_with("exports.")
                || text.starts_with("module.exports.")
        }),
        "arguments" => parent
            .parent()
            .and_then(|outer| outer.field("function"))
            .is_some_and(|callee| {
                let text = callee.text();
                text.ends_with("__exportStar") || text == "__export"
            }),
        _ => false,
    }
}

pub fn arguments<'r>(call: &SyntaxNode<'r>) -> Vec<SyntaxNode<'r>> {
    call.field("arguments")
        .map(|arguments| {
            arguments
                .children()
                .filter(|child| child.is_named())
                .collect()
        })
        .unwrap_or_default()
}

pub fn literal(node: &SyntaxNode) -> Option<String> {
    let text = node.text();
    match node.kind().as_ref() {
        "string" => Some(plain_text(&text)),
        "template_string" if !text.contains("${") => Some(plain_text(&text)),
        _ => None,
    }
}

pub fn unwrapped<'r>(node: SyntaxNode<'r>) -> SyntaxNode<'r> {
    let mut current = node;
    while WRAPPERS.contains(&current.kind().as_ref()) {
        let Some(inner) = current.children().filter(|child| child.is_named()).last() else {
            break;
        };
        current = inner;
    }
    current
}

pub fn specifiers(clause: &SyntaxNode) -> Vec<(String, String)> {
    clause
        .children()
        .filter(|child| child.kind() == "export_specifier")
        .filter_map(|specifier| {
            let name = plain(&specifier.field("name")?);
            let alias = specifier
                .field("alias")
                .map_or_else(|| name.clone(), |alias| plain(&alias));
            Some((name, alias))
        })
        .collect()
}

pub fn declared_names(declaration: &SyntaxNode) -> Vec<String> {
    match declaration.kind().as_ref() {
        "lexical_declaration" | "variable_declaration" => declaration
            .children()
            .filter(|child| child.kind() == "variable_declarator")
            .filter_map(|declarator| declarator.field("name"))
            .filter(|name| name.kind() == "identifier")
            .map(|name| name.text().to_string())
            .collect(),
        "identifier" => vec![declaration.text().to_string()],
        _ => declaration
            .field("name")
            .map(|name| plain(&name))
            .into_iter()
            .collect(),
    }
}

pub fn is_anonymous(declaration: &SyntaxNode) -> bool {
    declaration.kind() != "identifier"
        && declaration.kind() != "lexical_declaration"
        && declaration.field("name").is_none()
}

pub trait NodeChildren {
    fn has_child(&self, kind: &str) -> bool;
}

impl NodeChildren for SyntaxNode<'_> {
    fn has_child(&self, kind: &str) -> bool {
        self.children().any(|child| child.kind() == kind)
    }
}

pub fn plain(node: &SyntaxNode) -> String {
    plain_text(&node.text())
}

fn plain_text(text: &str) -> String {
    text.trim_matches(['"', '\'', '`']).to_string()
}
