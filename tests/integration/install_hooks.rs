use crate::support::{expected, scan_files, sightings, write_files};
use vigilante::{ScanRequest, Source};

const PACKAGE_MANIFEST: &str = r#"{
  "name": "app",
  "scripts": {
    "test": "vitest",
    "postinstall": "node -e \"require('fs').writeFileSync('postinstall-ran', '')\"",
    "prepare": "husky"
  }
}
"#;

const PRE_COMMIT_CONFIG: &str = "repos:
  - repo: https://github.com/psf/black
    rev: 24.1.0
    hooks:
      - id: black
  - repo: local
    hooks:
      - id: lint
        entry: ./scripts/lint.sh
  - repo: meta
    hooks:
      - id: check-hooks-apply
";

#[test]
fn install_and_git_hooks_are_found() {
    let map = scan_files(&[
        ("package.json", PACKAGE_MANIFEST),
        ("node_modules/left-pad/package.json", PACKAGE_MANIFEST),
        ("crates/core/Cargo.toml", "[package]\nname = \"core\"\n"),
        ("crates/core/build.rs", "fn main() {}\n"),
        ("src/build.rs", "pub fn build() {}\n"),
        ("tools/setup.py", "from setuptools import setup\nsetup()\n"),
        (".husky/pre-commit", "npx lint-staged\n"),
        (".husky/_/husky.sh", "#!/bin/sh\n"),
        (".pre-commit-config.yaml", PRE_COMMIT_CONFIG),
    ]);

    let postinstall = r#"node -e "require('fs').writeFileSync('postinstall-ran', '')""#;
    assert_eq!(
        sightings(&map),
        expected(&[
            &format!("build package.json install_hook:postinstall: {postinstall} package.json:5"),
            "build package.json install_hook:prepare: husky package.json:6",
            "build crates/core/build.rs install_hook:build.rs crates/core/build.rs:1",
            "build tools/setup.py install_hook:setup.py tools/setup.py:1",
            "build .husky/pre-commit git_hook:pre-commit .husky/pre-commit:1",
            "build .pre-commit-config.yaml git_hook:https://github.com/psf/black .pre-commit-config.yaml:2",
            "build .pre-commit-config.yaml git_hook:local: ./scripts/lint.sh .pre-commit-config.yaml:8",
        ])
    );
}

#[test]
fn scanning_never_runs_an_install_script() {
    let directory = write_files(&[("package.json", PACKAGE_MANIFEST)]);

    let map = vigilante::scan(ScanRequest {
        repository: directory.path().to_path_buf(),
        source: Source::WorkingDirectory,
    })
    .expect("scan");

    assert!(!map.is_empty());
    assert!(!directory.path().join("postinstall-ran").exists());
}
