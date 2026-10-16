use ast_grep_language::SupportLang;

use super::syntax::{self, loaded};
use crate::code::SyntaxNode;
use crate::deep::program::{FileIndex, Language, PrivilegedQuery};

const UNTRUSTED_SOURCES: &[&str] = &["params["];
const SHELL_CALLEES: &[&str] = &["system", "exec", "spawn"];

pub struct Ruby;

impl Language for Ruby {
    fn grammar(&self) -> SupportLang {
        SupportLang::Ruby
    }

    fn index(&self, root: &SyntaxNode) -> FileIndex {
        syntax::index(root)
    }

    fn privileged_modules(&self, query: PrivilegedQuery) -> Vec<String> {
        loaded(query.node).into_iter().collect()
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

    fn untrusted_sources(&self) -> &'static [&'static str] {
        UNTRUSTED_SOURCES
    }

    fn shell_callees(&self) -> &'static [&'static str] {
        SHELL_CALLEES
    }
}
