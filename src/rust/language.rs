use ast_grep_language::SupportLang;

use super::syntax::{self, leaves};
use crate::code::SyntaxNode;
use crate::deep::program::{FileIndex, Language};

pub struct Rust;

impl Language for Rust {
    fn grammar(&self) -> SupportLang {
        SupportLang::Rust
    }

    fn index(&self, root: &SyntaxNode) -> FileIndex {
        syntax::index(root)
    }

    fn privileged_module(&self, node: &SyntaxNode) -> Option<String> {
        if node.kind() != "use_declaration" {
            return None;
        }
        leaves(node)
            .iter()
            .map(|leaf| leaf.specifier())
            .max_by_key(String::len)
    }

    fn stem<'p>(&self, path: &'p str) -> &'p str {
        path.strip_suffix(".rs").unwrap_or(path)
    }

    fn imports_run_code(&self) -> bool {
        false
    }

    fn finds_members_by_type(&self) -> bool {
        true
    }
}
