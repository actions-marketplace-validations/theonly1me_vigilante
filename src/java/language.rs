use ast_grep_language::SupportLang;

use super::syntax::{self, import_of};
use crate::code::SyntaxNode;
use crate::deep::program::{FileIndex, Language};

pub struct Java;

impl Language for Java {
    fn grammar(&self) -> SupportLang {
        SupportLang::Java
    }

    fn index(&self, root: &SyntaxNode) -> FileIndex {
        syntax::index(root)
    }

    fn privileged_module(&self, node: &SyntaxNode) -> Option<String> {
        (node.kind() == "import_declaration")
            .then(|| import_of(node))
            .flatten()
            .map(|import| import.specifier)
    }

    fn stem<'p>(&self, path: &'p str) -> &'p str {
        path.strip_suffix(".java")
            .or_else(|| path.strip_suffix(".class"))
            .unwrap_or(path)
    }

    fn imports_run_code(&self) -> bool {
        false
    }
}
