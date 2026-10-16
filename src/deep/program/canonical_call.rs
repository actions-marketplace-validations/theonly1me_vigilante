use crate::code::SyntaxNode;

pub struct CanonicalCall<'r> {
    pub node: SyntaxNode<'r>,
    pub snippet: String,
}
