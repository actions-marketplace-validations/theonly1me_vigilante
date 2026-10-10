use ast_grep_language::SupportLang;

use super::syntax;
use crate::code::SyntaxNode;
use crate::deep::program::{FileIndex, Language};

pub struct Python;

impl Language for Python {
    fn grammar(&self) -> SupportLang {
        SupportLang::Python
    }

    fn index(&self, root: &SyntaxNode) -> FileIndex {
        syntax::index(root)
    }

    fn privileged_module(&self, node: &SyntaxNode) -> Option<String> {
        match node.kind().as_ref() {
            "dotted_name" => Some(node.text().to_string()),
            "import_statement" | "import_from_statement" => node
                .field("module_name")
                .or_else(|| node.field("name"))
                .map(|name| name.field("name").unwrap_or(name).text().to_string()),
            _ => None,
        }
    }

    fn stem<'p>(&self, path: &'p str) -> &'p str {
        path.strip_suffix(".py")
            .or_else(|| path.strip_suffix(".pyi"))
            .unwrap_or(path)
    }

    fn imports_run_code(&self) -> bool {
        true
    }

    fn finds_members_by_type(&self) -> bool {
        false
    }
}
