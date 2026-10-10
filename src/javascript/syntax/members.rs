use super::UseTarget;
use super::names::{is_assignment_target, is_called};
use crate::code::SyntaxNode;

pub struct MemberUse {
    pub target: UseTarget,
    pub skipped_child: Option<usize>,
}

pub fn member_use(node: &SyntaxNode) -> Option<MemberUse> {
    let object = node.field("object")?;
    let property = node
        .field("property")?
        .text()
        .trim_start_matches('#')
        .to_string();
    if is_assignment_target(node) {
        return None;
    }
    let call = is_called(node);
    match object.kind().as_ref() {
        "identifier" | "this" => Some(MemberUse {
            target: UseTarget::Member {
                object: object.text().to_string(),
                property,
                call,
            },
            skipped_child: Some(object.node_id()),
        }),
        _ if call => Some(MemberUse {
            target: UseTarget::Method(property),
            skipped_child: None,
        }),
        _ => None,
    }
}
