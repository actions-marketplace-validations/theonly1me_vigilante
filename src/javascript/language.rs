use ast_grep_language::SupportLang;

use super::paths::stem;
use super::syntax;
use crate::code::SyntaxNode;
use crate::deep::program::{CanonicalCall, FileIndex, Language, PrivilegedQuery};

pub struct JavaScript;

impl Language for JavaScript {
    fn grammar(&self) -> SupportLang {
        SupportLang::Tsx
    }

    fn index(&self, root: &SyntaxNode) -> FileIndex {
        syntax::index(root)
    }

    fn privileged_modules(&self, query: PrivilegedQuery) -> Vec<String> {
        let node = query.node;
        (node.kind() == "string")
            .then(|| node.text().trim_matches(['"', '\'', '`']).to_string())
            .into_iter()
            .collect()
    }

    fn stem<'p>(&self, path: &'p str) -> &'p str {
        stem(path)
    }

    fn imports_run_code(&self) -> bool {
        true
    }

    fn canonical_calls<'r>(&self, root: &SyntaxNode<'r>) -> Vec<CanonicalCall<'r>> {
        super::canonical::canonical_calls(root)
    }

    fn finds_members_by_type(&self) -> bool {
        false
    }
}
