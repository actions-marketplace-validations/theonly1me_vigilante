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
pub const SLOW_NODE: FakeTool = FakeTool {
    name: "node",
    script: "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n  echo v22.14.0\n  exit 0\nfi\nexec /bin/sleep 20\n",
};
pub const SLOW_DOCKER: FakeTool = FakeTool {
    name: "docker",
    script: "#!/bin/sh\ncase \"$1\" in\n  version) echo 27.0.0 ;;\n  pull | run) exec /bin/sleep 20 ;;\nesac\n",
};
pub const FAST_ENOUGH: Duration = Duration::from_secs(15);
pub const RUST_BASE: &[(&str, &str)] = &[
    (
        "Cargo.toml",
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    ),
    (
        "Cargo.lock",
        "version = 4\n\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\n",
    ),
    ("src/lib.rs", "pub fn build() {}\n"),
];
pub const RUST_HEAD: (&str, &str) = (
    "src/lib.rs",
    "pub fn build() {\n    std::process::Command::new(\"make\");\n}\n",
);
pub const RUST_OUT_OF_TIME: &str = "Vigilante did not build this Rust code (it reached the time limit of type_check_seconds = 1), so it resolved Rust calls by name only.";

pub struct FakeTool {
    pub name: &'static str,
    pub script: &'static str,
}

pub struct Project<'a> {
    pub engine: &'a str,
    pub base: &'a [(&'a str, &'a str)],
    pub head: (&'a str, &'a str),
    pub tool: FakeTool,
}

pub struct Timed {
    pub report: Value,
    pub elapsed: Duration,
}

pub fn review_with_slow_tool(project: Project) -> Timed {
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
    let tool = tools.path().join(project.tool.name);
    fs::write(&tool, project.tool.script).expect("tool script");
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).expect("executable");
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

pub fn reasons(report: &Value) -> Vec<String> {
    report["skipped"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|skip| skip["reason"].as_str().map(str::to_string))
        .collect()
}
