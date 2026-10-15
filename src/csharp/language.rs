use ast_grep_language::SupportLang;

use super::syntax;
use crate::code::SyntaxNode;
use crate::deep::program::{FileIndex, Language, PrivilegedQuery};

pub struct CSharp;

impl Language for CSharp {
    fn grammar(&self) -> SupportLang {
        SupportLang::CSharp
    }

    fn index(&self, root: &SyntaxNode) -> FileIndex {
        syntax::index(root)
    }

    fn privileged_modules(&self, query: PrivilegedQuery) -> Vec<String> {
        let node = query.node;
        (node.kind() == "using_directive")
            .then(|| {
                node.text()
                    .trim()
                    .trim_start_matches("global ")
                    .trim_start_matches("using")
                    .trim_end_matches(';')
                    .trim_start()
                    .trim_start_matches("static ")
                    .trim()
                    .to_string()
            })
            .into_iter()
            .collect()
    }

    fn stem<'p>(&self, path: &'p str) -> &'p str {
        path.strip_suffix(".cs").unwrap_or(path)
    }

    fn imports_run_code(&self) -> bool {
        false
    }

    fn finds_members_by_type(&self) -> bool {
        false
    }
}
