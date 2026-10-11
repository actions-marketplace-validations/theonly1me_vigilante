use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::time::{Duration, Instant};

use serde_json::Value;
use tempfile::TempDir;

use crate::npm_registry::report_of;
use crate::support::{Run, TestRepository};

const DIFF: &[&str] = &[
    "diff", "--base", "main", "--head", "feature", "--format", "json",
];
const SLOW_NODE: &str = "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n  echo v22.14.0\n  exit 0\nfi\nexec /bin/sleep 20\n";
const FAST_ENOUGH: Duration = Duration::from_secs(15);

struct Project<'a> {
    engine: &'a str,
    base: &'a [(&'a str, &'a str)],
    head: (&'a str, &'a str),
}

struct Timed {
    report: Value,
    elapsed: Duration,
}

fn review_with_slow_node(project: Project) -> Timed {
    let repository = TestRepository::new();
    let config = format!(
        "[engines]\n{} = \"typed\"\ntype_check_seconds = 1\n",
        project.engine
    );
    repository.write(project.base);
    repository.write(&[("vigilante.toml", &config)]);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(&[project.head]);
    repository.commit("change");
    let tools = TempDir::new().expect("tools");
    let node = tools.path().join("node");
    fs::write(&node, SLOW_NODE).expect("node script");
    fs::set_permissions(&node, fs::Permissions::from_mode(0o755)).expect("executable");
    let path = format!(
        "{}:{}",
        tools.path().display(),
        env::var("PATH").unwrap_or_default()
    );
    let cache = TempDir::new().expect("cache");
    let started = Instant::now();
    let output = Command::new(env!("CARGO_BIN_EXE_vigilante"))
        .arg("--repository")
        .arg(repository.directory.path())
        .args(DIFF)
        .env("XDG_CACHE_HOME", cache.path())
        .env("PATH", path)
        .output()
        .expect("vigilante runs");
    let elapsed = started.elapsed();
    Timed {
        report: report_of(&Run {
            code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }),
        elapsed,
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
#[ignore = "needs network"]
fn a_typescript_check_that_reaches_the_time_limit_falls_back_to_names() {
    let timed = review_with_slow_node(Project {
        engine: "javascript",
        base: &[
            ("package.json", "{\"name\": \"app\"}"),
            ("src/index.ts", "export function main() {\n  return 0;\n}\n"),
        ],
        head: (
            "src/index.ts",
            "import { execSync } from 'child_process';\n\nexport function main() {\n  return execSync('make');\n}\n",
        ),
    });

    assert!(
        reasons(&timed.report).contains(
            &"The TypeScript checker reached the time limit of type_check_seconds = 1, so Vigilante resolved calls by name only.".to_string()
        ),
        "{}",
        timed.report
    );
    assert!(timed.elapsed < FAST_ENOUGH, "{:?}", timed.elapsed);
}

#[test]
#[ignore = "needs network"]
fn a_python_check_that_reaches_the_time_limit_falls_back_to_names() {
    let timed = review_with_slow_node(Project {
        engine: "python",
        base: &[
            (
                "pyproject.toml",
                "[project]\nname = \"app\"\nversion = \"0.1.0\"\n",
            ),
            ("app.py", "def main():\n    return 0\n"),
        ],
        head: (
            "app.py",
            "import subprocess\n\n\ndef main():\n    return subprocess.run(['make'])\n",
        ),
    });

    assert!(
        reasons(&timed.report).contains(
            &"scip-python reached the time limit of type_check_seconds = 1, so Vigilante resolved Python calls by name only.".to_string()
        ),
        "{}",
        timed.report
    );
    assert!(timed.elapsed < FAST_ENOUGH, "{:?}", timed.elapsed);
}

#[test]
#[ignore = "needs docker"]
fn docker_a_rust_build_that_reaches_the_time_limit_falls_back_to_names() {
    let timed = review_with_slow_node(Project {
        engine: "rust",
        base: &[
            (
                "Cargo.toml",
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            ),
            (
                "Cargo.lock",
                "version = 4\n\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\n",
            ),
            ("src/lib.rs", "pub fn build() {}\n"),
        ],
        head: (
            "src/lib.rs",
            "pub fn build() {\n    std::process::Command::new(\"make\");\n}\n",
        ),
    });

    assert!(
        reasons(&timed.report).contains(
            &"Vigilante did not build this Rust code (it reached the time limit of type_check_seconds = 1), so it resolved Rust calls by name only.".to_string()
        ),
        "{}",
        timed.report
    );
    assert!(timed.elapsed < FAST_ENOUGH, "{:?}", timed.elapsed);
}
