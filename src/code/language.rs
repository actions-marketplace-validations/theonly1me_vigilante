use ast_grep_language::SupportLang;

pub const SUPPORTED_LANGUAGES: &[SupportLang] = &[
    SupportLang::CSharp,
    SupportLang::Go,
    SupportLang::Java,
    SupportLang::Python,
    SupportLang::Ruby,
    SupportLang::Rust,
    SupportLang::Tsx,
];

pub fn language_of(path: &str) -> Option<SupportLang> {
    let name = path.rsplit('/').next().unwrap_or(path);
    let (_, extension) = name.rsplit_once('.')?;
    match extension {
        "cs" => Some(SupportLang::CSharp),
        "go" => Some(SupportLang::Go),
        "java" => Some(SupportLang::Java),
        "py" | "pyi" => Some(SupportLang::Python),
        "rb" => Some(SupportLang::Ruby),
        "rs" => Some(SupportLang::Rust),
        "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" | "mts" | "cts" => Some(SupportLang::Tsx),
        _ => None,
    }
}
