use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

use crate::npm_registry::report_of;
use crate::python_fixtures::summaries;
use crate::python_index::{FakeWheel, PythonIndex};
use crate::support::{Run, TestRepository};

const DIFF: &[&str] = &[
    "diff", "--base", "main", "--head", "feature", "--format", "json",
];
const TOOLS: FakeWheel = FakeWheel {
    name: "tools",
    version: "1.0.0",
    files: &[(
        "tools/__init__.py",
        "import subprocess\n\n\nclass Runner:\n    def run(self, command):\n        return subprocess.run(command)\n\n\nclass Quiet:\n    def run(self, message):\n        return message\n",
    )],
};
const LOGGER: &str = "from tools import Runner\n\n\nclass Logger:\n    def run(self, message):\n        return message\n\n\ndef main(logger: Logger):\n    return logger.run('hi')\n";
const NAME_COLLISION: &str =
    ". exec tools app.py:10 app.main > tools/tools/__init__.Runner.run > subprocess.run";
const QUIET: &str =
    "from tools import Quiet\n\n\ndef main(quiet: Quiet):\n    return quiet.run('hi')\n";
const NO_NODE: &str = "Node.js is not on PATH, so Vigilante resolved Python calls by name only.";

struct Invocation<'a> {
    engine: &'a str,
    head: &'a str,
    path: Option<&'a str>,
}

fn run(invocation: Invocation) -> Run {
    let index = PythonIndex::serve(&[&TOOLS]);
    let repository = TestRepository::new();
    repository.write(&[
        (
            "pyproject.toml",
            "[project]\nname = \"app\"\nversion = \"0.1.0\"\n",
        ),
        (
            "vigilante.toml",
            &format!(
                "{}\n[engines]\npython = \"{}\"\n",
                index.allow_config(),
                invocation.engine
            ),
        ),
        ("uv.lock", &index.uv_lock(&[&TOOLS])),
        ("app.py", "def main():\n    return 0\n"),
    ]);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(&[("app.py", invocation.head)]);
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
#[ignore = "needs node and network"]
fn a_python_call_that_types_resolve_to_own_code_does_not_reach_a_package_method() {
    let report = report_of(&run(Invocation {
        engine: "typed",
        head: LOGGER,
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
fn the_python_name_based_engine_follows_every_method_of_the_same_name() {
    let report = report_of(&run(Invocation {
        engine: "deep",
        head: LOGGER,
        path: None,
    }));

    assert_eq!(
        summaries(&report["notes"]),
        vec![NAME_COLLISION, NAME_COLLISION]
    );
}

#[test]
#[ignore = "needs node and network"]
fn without_node_python_calls_resolve_by_name_and_the_scope_is_listed() {
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
        head: LOGGER,
        path: Some(&tools.path().to_string_lossy()),
    }));

    assert_eq!(
        summaries(&report["notes"]),
        vec![NAME_COLLISION, NAME_COLLISION]
    );
    assert_eq!(reasons(&report), vec![NO_NODE]);
}

#[test]
#[ignore = "needs node and network"]
fn a_typed_call_into_a_package_reaches_only_the_method_of_its_class() {
    let report = report_of(&run(Invocation {
        engine: "typed",
        head: QUIET,
        path: None,
    }));

    assert_eq!(reasons(&report), Vec::<String>::new());
    assert_eq!(
        summaries(&report["notes"]),
        Vec::<String>::new(),
        "{report}"
    );
}
