use ast_grep_language::SupportLang;

use super::syntax;
use crate::code::SyntaxNode;
use crate::deep::program::{CanonicalCall, FileIndex, Language, PrivilegedQuery};

const UNTRUSTED_SOURCES: &[&str] = &[
    "request.args",
    "request.form",
    "request.values",
    "request.json",
    "request.get_json(",
    "request.data",
    "request.files",
    "request.cookies",
    "request.headers",
    "request.GET",
    "request.POST",
    "request.query_params",
    "Query(",
    "Body(",
    "Form(",
    "Header(",
    "Cookie(",
];
const SHELL_CALLEES: &[&str] = &["system", "popen", "getoutput", "getstatusoutput"];

pub struct Python;

impl Language for Python {
    fn grammar(&self) -> SupportLang {
        SupportLang::Python
    }

    fn index(&self, root: &SyntaxNode) -> FileIndex {
        syntax::index(root)
    }

    fn privileged_modules(&self, query: PrivilegedQuery) -> Vec<String> {
        let node = query.node;
        match node.kind().as_ref() {
            "dotted_name" => vec![node.text().to_string()],
            "import_statement" | "import_from_statement" => node
                .field("module_name")
                .or_else(|| node.field("name"))
                .map(|name| name.field("name").unwrap_or(name).text().to_string())
                .into_iter()
                .collect(),
            _ => Vec::new(),
        }
    }

    fn stem<'p>(&self, path: &'p str) -> &'p str {
        path.strip_suffix(".py")
            .or_else(|| path.strip_suffix(".pyi"))
            .unwrap_or(path)
    }

    fn canonical_calls<'r>(&self, root: &SyntaxNode<'r>) -> Vec<CanonicalCall<'r>> {
        super::canonical::canonical_calls(root)
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
