mod file_filter;
mod language;
mod matches;

use std::collections::BTreeSet;

use ast_grep_config::{GlobalRules, RuleConfig, SerializableRuleConfig};
use ast_grep_core::tree_sitter::LanguageExt;
use ast_grep_language::SupportLang;

use crate::{Capability, Detection, Error, Power, SourceFile, Surface};
use file_filter::{FileFilter, RuleGlobs};
use language::SUPPORTED_LANGUAGES;
pub use language::language_of;
pub use matches::{MatchRequest, RuleMatch, SyntaxNode};

const BUILT_IN_PACKS: &[(&str, &str)] = &[
    ("packs/csharp.yml", include_str!("../../packs/csharp.yml")),
    ("packs/go.yml", include_str!("../../packs/go.yml")),
    ("packs/java.yml", include_str!("../../packs/java.yml")),
    (
        "packs/javascript_typescript.yml",
        include_str!("../../packs/javascript_typescript.yml"),
    ),
    ("packs/python.yml", include_str!("../../packs/python.yml")),
    ("packs/ruby.yml", include_str!("../../packs/ruby.yml")),
    ("packs/rust.yml", include_str!("../../packs/rust.yml")),
];

pub struct Rules {
    rules: Vec<Rule>,
    ids: BTreeSet<String>,
}

struct Rule {
    id: String,
    capability: Capability,
    inert: Vec<String>,
    language: SupportLang,
    files: FileFilter,
    config: RuleConfig<SupportLang>,
}

impl Rules {
    pub fn built_in() -> Result<Self, Error> {
        let mut rules = Self {
            rules: Vec::new(),
            ids: BTreeSet::new(),
        };
        for (path, text) in BUILT_IN_PACKS {
            rules.add(SourceFile { path, text })?;
        }
        Ok(rules)
    }

    pub fn add(&mut self, pack: SourceFile) -> Result<(), Error> {
        let invalid =
            |reason: String| Error::new(format!("invalid rule in {}: {reason}", pack.path));
        for document in yaml_documents(pack.text) {
            let serializable: SerializableRuleConfig<SupportLang> =
                ast_grep_config::from_str(document).map_err(|error| invalid(error.to_string()))?;
            let language = serializable.language;
            if !SUPPORTED_LANGUAGES.contains(&language) {
                return Err(invalid(format!("language {language} is not supported")));
            }
            let capability =
                capability_of(&serializable).map_err(|error| invalid(error.to_string()))?;
            let inert =
                inert_members_of(&serializable).map_err(|error| invalid(error.to_string()))?;
            let id = serializable.id.clone();
            let files = FileFilter::new(RuleGlobs {
                files: &serializable.files,
                ignores: &serializable.ignores,
            })
            .map_err(|error| invalid(format!("{id}: {error}")))?;
            if !self.ids.insert(id.clone()) {
                return Err(invalid(format!("rule id `{id}` is used twice")));
            }
            let config = RuleConfig::try_from(serializable, &GlobalRules::default())
                .map_err(|error| invalid(format!("{id}: {error}")))?;
            self.rules.push(Rule {
                id,
                capability,
                inert,
                language,
                files,
                config,
            });
        }
        Ok(())
    }

    pub fn handles(&self, path: &str) -> bool {
        language_of(path).is_some()
    }

    pub fn detect(&self, file: SourceFile) -> Vec<Detection> {
        let Some(language) = language_of(file.path) else {
            return Vec::new();
        };
        let rules: Vec<&Rule> = self
            .rules
            .iter()
            .filter(|rule| rule.language == language && rule.files.allows(file.path))
            .collect();
        if rules.is_empty() {
            return Vec::new();
        }
        let document = language.ast_grep(file.text);
        let root = document.root();
        rules
            .into_iter()
            .filter_map(|rule| {
                let found = root.find(&rule.config.matcher)?;
                Some(Detection {
                    power: Power::new(rule.capability),
                    line: found.start_pos().line() + 1,
                    rule: rule.id.clone(),
                })
            })
            .collect()
    }
}

fn inert_members_of(rule: &SerializableRuleConfig<SupportLang>) -> Result<Vec<String>, Error> {
    let metadata = serde_json::to_value(&rule.metadata)
        .map_err(|error| Error::new(format!("{}: {error}", rule.id)))?;
    let Some(inert) = metadata.get("inert") else {
        return Ok(Vec::new());
    };
    serde_json::from_value(inert.clone())
        .map_err(|error| Error::new(format!("{}: metadata.inert: {error}", rule.id)))
}

fn capability_of(rule: &SerializableRuleConfig<SupportLang>) -> Result<Capability, Error> {
    let metadata = serde_json::to_value(&rule.metadata)
        .map_err(|error| Error::new(format!("{}: {error}", rule.id)))?;
    let name = metadata
        .get("capability")
        .and_then(|capability| capability.as_str())
        .ok_or_else(|| Error::new(format!("{}: metadata.capability is missing", rule.id)))?;
    let capability: Capability = name.parse()?;
    if capability.surface() != Surface::Code {
        return Err(Error::new(format!(
            "{}: `{name}` is not a code capability",
            rule.id
        )));
    }
    Ok(capability)
}

fn yaml_documents(text: &str) -> Vec<&str> {
    let mut documents = Vec::new();
    let mut start = 0;
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        if line.trim_end() == "---" {
            documents.push(&text[start..offset]);
            start = offset + line.len();
        }
        offset += line.len();
    }
    documents.push(&text[start..]);
    documents
        .into_iter()
        .filter(|document| !document.trim().is_empty())
        .collect()
}
