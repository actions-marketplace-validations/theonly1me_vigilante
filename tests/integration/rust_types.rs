use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

use crate::npm_registry::report_of;
use crate::python_fixtures::summaries;
use crate::support::{Run, TestRepository};

const DIFF: &[&str] = &[
    "diff", "--base", "main", "--head", "feature", "--format", "json",
];
const MANIFEST: &str = "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ntools = { path = \"tools\" }\n";
const LOCKFILE: &str = "version = 4\n\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\ndependencies = [\n \"tools\",\n]\n\n[[package]]\nname = \"tools\"\nversion = \"0.1.0\"\n";
const TOOLS_MANIFEST: &str =
    "[package]\nname = \"tools\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
const TOOLS: &str = "pub struct Runner;\n\nimpl Runner {\n    pub fn run(&self) {\n        std::process::Command::new(\"make\");\n    }\n}\n";
const LOGGER: &str = "use tools::Runner;\n\npub struct Logger;\n\nimpl Logger {\n    pub fn run(&self) {}\n}\n\npub fn build(logger: &Logger, _runner: &Runner) {\n    logger.run();\n}\n";
const NAME_COLLISION: &str =
    ". exec tools src/lib.rs:10 src/lib.build > tools/src/lib.Runner.run > std::process::Command";

struct Invocation<'a> {
    engine: &'a str,
    path: Option<&'a str>,
}

fn run(invocation: Invocation) -> Run {
    let repository = TestRepository::new();
    repository.write(&[
        ("Cargo.toml", MANIFEST),
        ("Cargo.lock", LOCKFILE),
        ("tools/Cargo.toml", TOOLS_MANIFEST),
        ("tools/src/lib.rs", TOOLS),
        (
            "vigilante.toml",
            &format!("[engines]\nrust = \"{}\"\n", invocation.engine),
        ),
        ("src/lib.rs", "pub fn build() {}\n"),
    ]);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(&[("src/lib.rs", LOGGER)]);
    repository.commit("change");
    let cache = TempDir::new().expect("cache");
    let mut command = Command::new(env!("CARGO_BIN_EXE_vigilante"));
    command
        .arg("--repository")
        .arg(repository.directory.path())
        .args(DIFF)
        .env("XDG_CACHE_HOME", cache.path());
    if let Some(path) = invocation.path {
        command.env("PATH", path);
    }
    let output = command.output().expect("vigilante runs");
    Run {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn reasons(report: &Value) -> Vec<String> {
    report["skipped"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|skip| skip["reason"].as_str().map(str::to_string))
        .collect()
}

#[test]
#[ignore = "needs docker"]
fn docker_a_rust_call_that_types_resolve_to_own_code_does_not_reach_a_crate_method() {
    let report = report_of(&run(Invocation {
        engine: "typed",
        path: None,
    }));

    assert_eq!(reasons(&report), Vec::<String>::new());
    assert_eq!(
        summaries(&report["notes"]),
        Vec::<String>::new(),
        "{report}"
    );
}

#[test]
fn the_rust_name_based_engine_follows_every_method_of_the_same_name() {
    let report = report_of(&run(Invocation {
        engine: "deep",
        path: None,
    }));

    assert_eq!(summaries(&report["notes"]), vec![NAME_COLLISION]);
}

#[test]
fn without_docker_rust_calls_resolve_by_name_and_the_scope_is_listed() {
    let tools = TempDir::new().expect("tools");
    let git = Command::new("sh")
        .args(["-c", "command -v git"])
        .output()
        .expect("git path");
    std::os::unix::fs::symlink(
        String::from_utf8_lossy(&git.stdout).trim(),
        tools.path().join("git"),
    )
    .expect("git link");
    let report = report_of(&run(Invocation {
        engine: "typed",
        path: Some(&tools.path().to_string_lossy()),
    }));

    assert_eq!(summaries(&report["notes"]), vec![NAME_COLLISION]);
    assert_eq!(
        reasons(&report),
        vec![
            "Vigilante did not build this Rust code (Docker is not available), so it resolved Rust calls by name only."
        ]
    );
}
