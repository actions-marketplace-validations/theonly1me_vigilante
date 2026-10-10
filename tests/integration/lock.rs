use std::fs;

use vigilante::Lock;

use crate::cli::MCP_SERVER;
use crate::support::{TestRepository, scan_files};

const EXPECTED_LOCK: &str = r#"version = 1

[code.services]
powers = [
    "exec",
    "network",
]

[agent.".mcp.json"]
powers = ["mcp_server:fs-tools: npx -y @acme/fs-mcp"]

[build."web/package.json"]
powers = ["install_hook:postinstall: node setup.js"]
"#;

fn fixture() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "services/billing/export.py",
            "import requests, subprocess; subprocess.run(['true']); requests.get('https://example.com')\n",
        ),
        (
            ".mcp.json",
            r#"{"mcpServers": {"fs-tools": {"command": "npx", "args": ["-y", "@acme/fs-mcp"]}}}"#,
        ),
        (
            "web/package.json",
            r#"{"scripts": {"postinstall": "node setup.js"}}"#,
        ),
    ]
}

#[test]
fn the_lock_lists_each_scope_with_sorted_powers() {
    let lock = Lock::of(&scan_files(&fixture()));

    assert_eq!(lock.render(), EXPECTED_LOCK);
}

#[test]
fn a_rendered_lock_parses_back_to_the_same_powers() {
    let lock = Lock::of(&scan_files(&fixture()));

    assert_eq!(Lock::parse(&lock.render()), Ok(lock));
}

#[test]
fn a_lock_with_another_version_is_rejected() {
    let error = Lock::parse("version = 2\n").expect_err("version 2 is not supported");

    assert!(error.to_string().contains("version 2"), "{error}");
}

#[test]
fn lock_writes_the_same_bytes_every_time() {
    let repository = TestRepository::new();
    repository.write(&[
        (
            "web/app.ts",
            "const key = process.env.KEY;\nawait fetch(url);\n",
        ),
        (
            "services/run.py",
            "import socket, subprocess; subprocess.run(['true']); socket.socket()\n",
        ),
        ("crates/core/src/lib.rs", "use std::process::Command;\n"),
        (".mcp.json", MCP_SERVER),
    ]);

    let run = repository.vigilante(&["lock"]);
    let first = fs::read(repository.directory.path().join("vigilante.lock")).expect("lock");
    repository.vigilante(&["lock"]);
    let second = fs::read(repository.directory.path().join("vigilante.lock")).expect("lock");

    assert_eq!(
        run.stdout,
        "Wrote vigilante.lock with 5 powers in 4 scopes.\n"
    );
    assert_eq!(first, second);
}

#[test]
fn lock_counts_a_single_power_in_the_singular() {
    let repository = TestRepository::new();
    repository.write(&[(
        "services/run.py",
        "import subprocess; subprocess.run(['true'])\n",
    )]);

    let run = repository.vigilante(&["lock"]);

    assert_eq!(
        run.stdout,
        "Wrote vigilante.lock with 1 power in 1 scope.\n"
    );
}
