use ast_grep_language::SupportLang;

use super::syntax::{self, import_of};
use crate::code::SyntaxNode;
use crate::deep::program::{CanonicalCall, FileIndex, Language, PrivilegedQuery};

const UNTRUSTED_SOURCES: &[&str] = &[
    "getParameter(",
    "getHeader(",
    "getQueryString(",
    "getInputStream(",
    "@RequestParam",
    "@PathVariable",
    "@RequestBody",
    "@RequestHeader",
    "@CookieValue",
];

pub struct Java;

impl Language for Java {
    fn grammar(&self) -> SupportLang {
        SupportLang::Java
    }

    fn index(&self, root: &SyntaxNode) -> FileIndex {
        syntax::index(root)
    }

    fn privileged_modules(&self, query: PrivilegedQuery) -> Vec<String> {
        (query.node.kind() == "import_declaration")
            .then(|| import_of(query.node))
            .flatten()
            .map(|import| import.specifier)
            .into_iter()
            .collect()
    }

    fn stem<'p>(&self, path: &'p str) -> &'p str {
        path.strip_suffix(".java")
            .or_else(|| path.strip_suffix(".class"))
            .unwrap_or(path)
    }

    fn imports_run_code(&self) -> bool {
        false
    }

    fn canonical_calls<'r>(&self, root: &SyntaxNode<'r>) -> Vec<CanonicalCall<'r>> {
        super::canonical::canonical_calls(root)
    }

    fn finds_members_by_type(&self) -> bool {
        false
    }

    fn untrusted_sources(&self) -> &'static [&'static str] {
        UNTRUSTED_SOURCES
    }
}
