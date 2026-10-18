use serde_json::Value;

use crate::support::TestRepository;

const SPAWN: &str = "const { spawn } = require('child_process');\nspawn('ls');\n";
const PACKAGE: &str = r#"{"name": "server", "version": "1.0.0"}"#;

fn repository_with(paths: &[&str]) -> TestRepository {
    let repository = TestRepository::new();
    repository.write(&[("package.json", PACKAGE)]);
    for path in paths {
        repository.write(&[(path, SPAWN)]);
    }
    repository
}

fn evidence_paths(repository: &TestRepository) -> Vec<String> {
    let run = repository.vigilante(&["scan", "--format", "json"]);
    assert_eq!(run.code, 0, "{}", run.stderr);
    let report: Value = serde_json::from_str(&run.stdout).expect("json");
    report["powers"][0]["powers"][0]["evidence"]
        .as_array()
        .expect("evidence array")
        .iter()
        .map(|item| item["path"].as_str().expect("path").to_string())
        .collect()
}

#[test]
fn scan_json_keeps_the_first_example_and_adds_one_from_each_other_folder() {
    let repository = repository_with(&["scripts/a.js", "src/b.js", "src/c.js"]);

    assert_eq!(
        evidence_paths(&repository),
        vec!["scripts/a.js", "src/b.js"]
    );
}

#[test]
fn scan_text_prints_an_example_from_each_folder() {
    let repository = repository_with(&["scripts/a.js", "src/b.js", "src/c.js"]);

    let run = repository.vigilante(&["scan"]);

    let rows: Vec<&str> = run.stdout.lines().collect();
    assert_eq!(rows.len(), 2, "{}", run.stdout);
    assert!(rows[0].ends_with("scripts/a.js:2"), "{}", run.stdout);
    assert!(rows[1].ends_with("src/b.js:2"), "{}", run.stdout);
}

#[test]
fn scan_shows_three_examples_at_most() {
    let repository = repository_with(&["a/x.js", "b/x.js", "c/x.js", "d/x.js"]);

    assert_eq!(
        evidence_paths(&repository),
        vec!["a/x.js", "b/x.js", "c/x.js"]
    );
}

#[test]
fn scan_shows_one_example_when_the_power_is_in_one_folder() {
    let repository = repository_with(&["src/b.js", "src/c.js"]);

    assert_eq!(evidence_paths(&repository), vec!["src/b.js"]);
}

#[test]
fn scan_json_lists_the_shortest_example_first_even_when_it_sorts_later() {
    let repository = TestRepository::new();
    repository.write(&[
        ("package.json", PACKAGE),
        (
            "a/entry.js",
            "const { run } = require('../b/helper');\nrun();\n",
        ),
        (
            "b/helper.js",
            "const { spawn } = require('child_process');\nfunction run() { spawn('ls'); }\nmodule.exports = { run };\n",
        ),
    ]);

    assert_eq!(
        evidence_paths(&repository),
        vec!["b/helper.js", "a/entry.js"]
    );
}

#[test]
fn scan_prints_the_same_bytes_on_every_run() {
    let repository = repository_with(&["scripts/a.js", "src/b.js", "lib/c.js"]);

    for format in ["text", "json"] {
        let first = repository.vigilante(&["scan", "--format", format]);
        let second = repository.vigilante(&["scan", "--format", format]);
        assert_eq!(first.stdout, second.stdout);
    }
}
