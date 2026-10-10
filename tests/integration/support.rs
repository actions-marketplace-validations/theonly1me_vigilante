use std::collections::BTreeSet;
use std::fs;
use std::process::Command;

use tempfile::TempDir;
use vigilante::{PowerMap, Review, ReviewRequest, ScanRequest, Source};

pub fn write_files(files: &[(&str, &str)]) -> TempDir {
    let directory = tempfile::tempdir().expect("temporary directory");
    for (path, text) in files {
        let target = directory.path().join(path);
        fs::create_dir_all(target.parent().expect("parent folder")).expect("folders");
        fs::write(target, text).expect("file");
    }
    directory
}

pub fn scan_files(files: &[(&str, &str)]) -> PowerMap {
    let directory = write_files(files);
    vigilante::scan(ScanRequest {
        repository: directory.path().to_path_buf(),
        source: Source::WorkingDirectory,
    })
    .expect("scan")
}

pub fn sightings(map: &PowerMap) -> BTreeSet<String> {
    map.scopes
        .iter()
        .flat_map(|(scope, powers)| {
            powers.iter().map(move |(power, evidence)| {
                let first = evidence.first().expect("evidence");
                format!(
                    "{} {} {power} {}:{}",
                    scope.surface.name(),
                    scope.path,
                    first.path,
                    first.line
                )
            })
        })
        .collect()
}

pub fn expected(lines: &[&str]) -> BTreeSet<String> {
    lines.iter().map(|line| line.to_string()).collect()
}

pub struct TestRepository {
    pub directory: TempDir,
}

impl TestRepository {
    pub fn new() -> Self {
        let repository = Self {
            directory: tempfile::tempdir().expect("temporary directory"),
        };
        repository.git(&["init", "--quiet", "--initial-branch=main"]);
        repository
    }

    pub fn write(&self, files: &[(&str, &str)]) {
        for (path, text) in files {
            let target = self.directory.path().join(path);
            fs::create_dir_all(target.parent().expect("parent folder")).expect("folders");
            fs::write(target, text).expect("file");
        }
    }

    pub fn remove(&self, path: &str) {
        fs::remove_file(self.directory.path().join(path)).expect("removed file");
    }

    pub fn commit(&self, message: &str) -> String {
        self.git(&["add", "--all"]);
        self.git(&["commit", "--quiet", "--allow-empty", "--message", message]);
        self.git(&["rev-parse", "HEAD"])
    }

    pub fn git(&self, arguments: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(self.directory.path())
            .args(arguments)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_AUTHOR_NAME", "Test")
            .env("GIT_AUTHOR_EMAIL", "test@example.com")
            .env("GIT_COMMITTER_NAME", "Test")
            .env("GIT_COMMITTER_EMAIL", "test@example.com")
            .output()
            .expect("git runs");
        assert!(
            output.status.success(),
            "git {arguments:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    pub fn diff(&self, branches: Branches) -> Review {
        vigilante::review(ReviewRequest {
            repository: self.directory.path().to_path_buf(),
            base: Some(branches.base.to_string()),
            head: Source::Commit(branches.head.to_string()),
        })
        .expect("review")
    }

    pub fn vigilante(&self, arguments: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_vigilante"))
            .arg("--repository")
            .arg(self.directory.path())
            .args(arguments)
            .output()
            .expect("vigilante runs");
        Run {
            code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    pub fn check(&self) -> Review {
        vigilante::review(ReviewRequest {
            repository: self.directory.path().to_path_buf(),
            base: None,
            head: Source::WorkingDirectory,
        })
        .expect("review")
    }
}

pub struct Run {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub struct Branches<'a> {
    pub base: &'a str,
    pub head: &'a str,
}

pub fn finding_lines(review: &Review) -> Vec<String> {
    review
        .findings
        .iter()
        .map(|finding| {
            format!(
                "{} {} {} locked={} denied={}",
                finding.scope.surface.name(),
                finding.scope.path,
                finding.power,
                finding.locked,
                finding.denied
            )
        })
        .collect()
}
