use std::fs;

use crate::support::TestRepository;

const MCP_SERVER: &str =
    r#"{"mcpServers": {"fs-tools": {"command": "npx", "args": ["-y", "@acme/fs-mcp"]}}}"#;

const WRITE_WORKFLOW: &str = "on: push\npermissions:\n  contents: write\njobs: {}\n";

const MARKER_SCRIPT: &str = r#"{"scripts": {"postinstall": "node -e \"require('fs').writeFileSync('postinstall-ran', '')\""}}"#;

fn rows_with(output: &str, status: &str) -> usize {
    output
        .lines()
        .filter(|line| line.starts_with("| ") && line.ends_with(&format!("{status} |")))
        .count()
}

#[test]
fn diff_exits_1_until_the_lock_is_committed_and_then_exits_0() {
    let repository = TestRepository::new();
    repository.write(&[("README.md", "# app\n")]);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(&[
        (".mcp.json", MCP_SERVER),
        (".github/workflows/release.yml", WRITE_WORKFLOW),
        ("services/billing/export.py", "import subprocess\n"),
    ]);
    repository.commit("add powers");

    let before = repository.vigilante(&["diff", "--base", "main", "--head", "feature"]);
    let lock = repository.vigilante(&["lock"]);
    repository.commit("lock powers");
    let after = repository.vigilante(&["diff", "--base", "main", "--head", "feature"]);

    assert_eq!(before.code, 1, "{}", before.stderr);
    assert!(
        before
            .stdout
            .starts_with("## Vigilante: this pull request adds 3 new powers\n")
    );
    assert_eq!(
        rows_with(&before.stdout, "**not in lock**"),
        3,
        "{}",
        before.stdout
    );
    assert_eq!(
        lock.stdout,
        "Wrote vigilante.lock with 3 powers in 3 scopes.\n"
    );
    assert_eq!(after.code, 0, "{}", after.stdout);
    assert_eq!(rows_with(&after.stdout, "in lock"), 3, "{}", after.stdout);
    assert!(after.stdout.ends_with("**Result: pass.**\n"));
}

#[test]
fn lock_writes_the_same_bytes_every_time() {
    let repository = TestRepository::new();
    repository.write(&[
        (
            "web/app.ts",
            "const key = process.env.KEY;\nawait fetch(url);\n",
        ),
        ("services/run.py", "import subprocess\nimport socket\n"),
        ("crates/core/src/lib.rs", "use std::process::Command;\n"),
        (".mcp.json", MCP_SERVER),
    ]);

    repository.vigilante(&["lock"]);
    let first = fs::read(repository.directory.path().join("vigilante.lock")).expect("lock");
    repository.vigilante(&["lock"]);
    let second = fs::read(repository.directory.path().join("vigilante.lock")).expect("lock");

    assert!(!first.is_empty());
    assert_eq!(first, second);
}

#[test]
fn check_exits_1_for_a_stale_lock_entry() {
    let repository = TestRepository::new();
    repository.write(&[
        ("services/run.py", "value = 1\n"),
        (
            "vigilante.lock",
            "version = 1\n\n[code.services]\npowers = [\"network\"]\n",
        ),
    ]);

    let run = repository.vigilante(&["check"]);

    assert_eq!(run.code, 1);
    assert!(
        run.stdout.contains("1 lock entry is stale."),
        "{}",
        run.stdout
    );
}

#[test]
fn errors_exit_with_code_2() {
    let repository = TestRepository::new();
    repository.write(&[("README.md", "# app\n")]);
    repository.commit("base");

    let run = repository.vigilante(&["diff", "--base", "missing-branch"]);

    assert_eq!(run.code, 2);
    assert!(
        run.stderr.contains("`missing-branch` is not a commit"),
        "{}",
        run.stderr
    );
}

#[test]
fn scan_prints_each_power_and_never_runs_install_scripts() {
    let repository = TestRepository::new();
    repository.write(&[
        ("services/run.py", "import subprocess\n"),
        ("web/package.json", MARKER_SCRIPT),
    ]);

    let run = repository.vigilante(&["scan"]);

    assert_eq!(run.code, 0);
    let exec_line = run
        .stdout
        .lines()
        .find(|line| line.contains(" exec "))
        .expect("exec line");
    assert!(exec_line.starts_with("code   services "), "{exec_line}");
    assert!(exec_line.ends_with("services/run.py:1"), "{exec_line}");
    assert!(run.stdout.contains("install_hook:postinstall"));
    assert!(
        !repository
            .directory
            .path()
            .join("web/postinstall-ran")
            .exists()
    );
    assert!(!repository.directory.path().join("postinstall-ran").exists());
}
