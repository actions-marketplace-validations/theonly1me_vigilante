use ast_grep_language::SupportLang;

use super::paths::stem;
use super::syntax;
use crate::code::SyntaxNode;
use crate::deep::program::{FileIndex, Language};

pub struct JavaScript;

impl Language for JavaScript {
    fn grammar(&self) -> SupportLang {
        SupportLang::Tsx
    }

    fn index(&self, root: &SyntaxNode) -> FileIndex {
        syntax::index(root)
    }

    fn privileged_module(&self, node: &SyntaxNode) -> Option<String> {
        (node.kind() == "string").then(|| node.text().trim_matches(['"', '\'', '`']).to_string())
    }

    fn stem<'p>(&self, path: &'p str) -> &'p str {
        stem(path)
    }
}
