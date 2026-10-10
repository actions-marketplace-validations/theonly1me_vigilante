use ast_grep_language::SupportLang;

use super::syntax::{self, loaded};
use crate::code::SyntaxNode;
use crate::deep::program::{FileIndex, Language};

pub struct Ruby;

impl Language for Ruby {
    fn grammar(&self) -> SupportLang {
        SupportLang::Ruby
    }

    fn index(&self, root: &SyntaxNode) -> FileIndex {
        syntax::index(root)
    }

    fn privileged_module(&self, node: &SyntaxNode) -> Option<String> {
        loaded(node)
    }

    fn stem<'p>(&self, path: &'p str) -> &'p str {
        path.strip_suffix(".rb").unwrap_or(path)
    }

    fn imports_run_code(&self) -> bool {
        true
    }

    fn finds_members_by_type(&self) -> bool {
        false
    }
}
