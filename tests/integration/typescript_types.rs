use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

use crate::javascript_fixtures::notes;
use crate::npm_registry::{FakePackage, LockEntry, NpmRegistry, report_of};
use crate::support::{Run, TestRepository};

const DIFF: &[&str] = &[
    "diff", "--base", "main", "--head", "feature", "--format", "json",
];
const MANIFEST: (&str, &str) = (
    "package.json",
    "{\"name\": \"app\", \"dependencies\": {\"tools\": \"^1.0.0\"}}",
);
const TOOLS: FakePackage = FakePackage {
    name: "tools",
    version: "1.0.0",
    files: &[(
        "index.js",
        "const cp = require('child_process');\n\nclass Runner {\n  run(command) {\n    return cp.execSync(command);\n  }\n}\n\nmodule.exports = { Runner };\n",
    )],
};
const LOGGER: &str = "import { Runner } from 'tools';\n\nclass Logger {\n  run(message: string) {\n    return message.length;\n  }\n}\n\nexport function main(logger: Logger) {\n  return logger.run('hi');\n}\n";
const CALLBACK: &str = "export function each(callback: () => number) {\n  return callback();\n}\n";
const NAME_COLLISION: &str = ". exec through tools src/index.ts:10 src/index.main > tools/index.Runner.run > child_process.execSync";
const EMPTY: &str = "export function main() {\n  return 0;\n}\n";

struct Change<'a> {
    engine: &'a str,
    head: &'a str,
}

fn review(change: Change) -> Value {
    report_of(&run(Invocation { change, path: None }))
}

struct Invocation<'a> {
    change: Change<'a>,
    path: Option<&'a str>,
}

fn run(input: Invocation) -> Run {
    let registry = NpmRegistry::serve(&[&TOOLS]);
    let repository = TestRepository::new();
    repository.write(&[
        MANIFEST,
        (
            "vigilante.toml",
            &format!(
                "{}\n[engines]\njavascript = \"{}\"\n",
                registry.allow_config(),
                input.change.engine
            ),
        ),
        (
            "package-lock.json",
            &registry.lockfile(&[LockEntry {
                key: "node_modules/tools",
                package: &TOOLS,
            }]),
        ),
        ("src/index.ts", EMPTY),
    ]);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(&[("src/index.ts", input.change.head)]);
    repository.commit("change");
    let cache = TempDir::new().expect("cache");
    let mut command = Command::new(env!("CARGO_BIN_EXE_vigilante"));
    command
        .arg("--repository")
        .arg(repository.directory.path())
        .args(DIFF)
        .env("XDG_CACHE_HOME", cache.path());
    if let Some(path) = input.path {
        command.env("PATH", path);
    }
    let output = command.output().expect("vigilante runs");
    Run {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

#[test]
#[ignore = "needs node and network"]
fn a_call_that_types_resolve_to_own_code_does_not_reach_a_package_method_of_the_same_name() {
    let report = review(Change {
        engine: "typed",
        head: LOGGER,
    });

    assert_eq!(notes(&report), Vec::<String>::new(), "{report}");
}

#[test]
fn the_name_based_engine_follows_a_call_to_every_method_of_the_same_name() {
    let report = review(Change {
        engine: "deep",
        head: LOGGER,
    });

    assert_eq!(notes(&report), vec![NAME_COLLISION, NAME_COLLISION]);
}

#[test]
#[ignore = "needs node and network"]
fn a_call_through_a_function_value_is_an_unanalyzed_note() {
    let report = review(Change {
        engine: "typed",
        head: CALLBACK,
    });

    assert_eq!(
        notes(&report),
        vec![
            ". unanalyzed through . src/index.ts:2 src/index.each > a call that types cannot follow"
        ]
    );
}

#[test]
#[ignore = "needs node and network"]
fn without_node_the_review_resolves_by_name_and_lists_the_scope() {
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
    let run = run(Invocation {
        change: Change {
            engine: "typed",
            head: LOGGER,
        },
        path: Some(&tools.path().to_string_lossy()),
    });
    let report = report_of(&run);

    assert_eq!(notes(&report), vec![NAME_COLLISION, NAME_COLLISION]);
    assert!(
        report["skipped"]
            .as_array()
            .expect("skipped")
            .iter()
            .any(|skip| skip["reason"]
                == "Node.js is not on PATH, so Vigilante resolved JavaScript and TypeScript calls by name only."),
        "{report}"
    );
}
