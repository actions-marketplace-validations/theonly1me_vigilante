use ast_grep_language::SupportLang;

use super::syntax::{self, Leaf, SEPARATOR, leaves};
use crate::code::SyntaxNode;
use crate::deep::program::{CanonicalCall, FileIndex, Language, PrivilegedQuery};

pub struct Rust;

impl Language for Rust {
    fn grammar(&self) -> SupportLang {
        SupportLang::Rust
    }

    fn index(&self, root: &SyntaxNode) -> FileIndex {
        syntax::index(root)
    }

    fn privileged_modules(&self, query: PrivilegedQuery) -> Vec<String> {
        if query.node.kind() != "use_declaration" {
            return Vec::new();
        }
        leaves(query.node)
            .iter()
            .filter(|leaf| (query.is_privileged)(&format!("use {};", leaf.path.join(SEPARATOR))))
            .map(Leaf::specifier)
            .collect()
    }

    fn stem<'p>(&self, path: &'p str) -> &'p str {
        path.strip_suffix(".rs").unwrap_or(path)
    }

    fn imports_run_code(&self) -> bool {
        false
    }

    fn canonical_calls<'r>(&self, root: &SyntaxNode<'r>) -> Vec<CanonicalCall<'r>> {
        super::canonical::canonical_calls(root)
    }

    fn finds_members_by_type(&self) -> bool {
        true
    }
}
