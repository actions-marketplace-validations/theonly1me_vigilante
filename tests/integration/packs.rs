use std::collections::BTreeSet;
use std::fs;

use crate::support::write_files;
use saphyr::{LoadableYamlNode, Yaml};
use vigilante::{ScanRequest, Source};

const PACKS: &[(&str, &str)] = &[
    ("csharp", "cs"),
    ("go", "go"),
    ("java", "java"),
    ("javascript_typescript", "ts"),
    ("python", "py"),
    ("ruby", "rb"),
    ("rust", "rs"),
];

struct Case {
    rule: String,
    snippet: String,
    must_match: bool,
}

fn yaml_documents(path: &str) -> Vec<Yaml<'static>> {
    let text = fs::read_to_string(path).expect("pack file");
    Yaml::load_from_str(Box::leak(text.into_boxed_str())).expect("valid YAML")
}

fn text_of<'a>(document: &'a Yaml, key: &str) -> &'a str {
    document
        .as_mapping_get(key)
        .and_then(|value| value.as_str())
        .unwrap_or_default()
}

fn snippets_of<'a>(document: &'a Yaml, key: &str) -> Vec<&'a str> {
    document
        .as_mapping_get(key)
        .and_then(|value| value.as_sequence())
        .map(|items| items.iter().filter_map(|item| item.as_str()).collect())
        .unwrap_or_default()
}

fn cases_of(language: &str) -> Vec<Case> {
    let mut cases = Vec::new();
    for document in yaml_documents(&format!("packs/tests/{language}.yml")) {
        let rule = text_of(&document, "id");
        for (key, must_match) in [("invalid", true), ("valid", false)] {
            for snippet in snippets_of(&document, key) {
                cases.push(Case {
                    rule: rule.to_string(),
                    snippet: snippet.to_string(),
                    must_match,
                });
            }
        }
    }
    cases
}

fn rule_ids_with_match_cases(language: &str) -> BTreeSet<String> {
    yaml_documents(&format!("packs/tests/{language}.yml"))
        .iter()
        .filter(|document| !snippets_of(document, "invalid").is_empty())
        .map(|document| text_of(document, "id").to_string())
        .collect()
}

fn pack_rule_ids(language: &str) -> BTreeSet<String> {
    yaml_documents(&format!("packs/{language}.yml"))
        .iter()
        .map(|document| text_of(document, "id").to_string())
        .collect()
}

#[test]
fn each_rule_matches_its_invalid_cases_and_no_valid_case() {
    let mut failures = Vec::new();
    for (language, extension) in PACKS {
        let cases = cases_of(language);
        let paths: Vec<String> = (0..cases.len())
            .map(|index| format!("snippets/case_{index}.{extension}"))
            .collect();
        let files: Vec<(&str, &str)> = paths
            .iter()
            .zip(&cases)
            .map(|(path, case)| (path.as_str(), case.snippet.as_str()))
            .collect();
        let directory = write_files(&files);
        let map = vigilante::scan(ScanRequest {
            repository: directory.path().to_path_buf(),
            source: Source::WorkingDirectory,
        })
        .expect("scan");
        let matched: BTreeSet<(&str, &str)> = map
            .scopes
            .values()
            .flat_map(|powers| powers.values().flatten())
            .map(|evidence| (evidence.path.as_str(), evidence.rule.as_str()))
            .collect();
        for (path, case) in paths.iter().zip(&cases) {
            if matched.contains(&(path.as_str(), case.rule.as_str())) != case.must_match {
                let expectation = if case.must_match {
                    "must match"
                } else {
                    "must not match"
                };
                failures.push(format!("{} {expectation}: {:?}", case.rule, case.snippet));
            }
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

#[test]
fn each_pack_rule_has_a_case_that_it_must_match() {
    for (language, _) in PACKS {
        assert_eq!(
            pack_rule_ids(language),
            rule_ids_with_match_cases(language),
            "{language}"
        );
    }
}
