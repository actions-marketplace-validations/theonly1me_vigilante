use vigilante::Lock;

use crate::support::scan_files;

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
            "import subprocess\nimport requests\n",
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
