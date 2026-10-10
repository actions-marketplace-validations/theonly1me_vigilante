use crate::code::SyntaxNode;

pub fn is_test_item(node: &SyntaxNode) -> bool {
    node.prev_all()
        .take_while(|sibling| {
            matches!(
                sibling.kind().as_ref(),
                "attribute_item" | "line_comment" | "block_comment"
            )
        })
        .any(|sibling| is_test_attribute(&sibling.text()))
}

fn is_test_attribute(text: &str) -> bool {
    let compact: String = text
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    compact == "#[test]"
        || compact.starts_with("#[cfg(test")
        || compact.starts_with("#[cfg(all(test")
        || compact.contains("::test]")
        || compact.contains("::test(")
}

pub fn test_module_names(root: &SyntaxNode) -> Vec<String> {
    root.children()
        .filter(|node| node.kind() == "mod_item" && is_test_item(node))
        .filter_map(|node| node.field("name").map(|name| name.text().to_string()))
        .collect()
}
