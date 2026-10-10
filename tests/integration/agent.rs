use crate::support::{expected, scan_files, sightings};

const CLAUDE_SETTINGS: &str = r#"{
  "permissions": {
    "allow": ["Bash(npm run test:*)", "WebFetch"],
    "deny": ["Bash(rm:*)"],
    "defaultMode": "bypassPermissions"
  },
  "enableAllProjectMcpServers": true,
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Edit",
        "hooks": [
          { "type": "command", "command": "./scripts/lint.sh" }
        ]
      }
    ]
  }
}
"#;

const MCP_SERVERS_WITH_COMMENTS: &str = r#"{
  // project servers, see https://example.com/docs
  "mcpServers": {
    "fs-tools": { "command": "npx", "args": ["-y", "@acme/fs-mcp"] },
    "docs": { "type": "http", "url": "https://docs.example.com/mcp" }
  }
}
"#;

const CODEX_CONFIG: &str = r#"approval_policy = "never"
sandbox_mode = "workspace-write"

[mcp_servers.search]
command = "uvx"
args = ["search-mcp"]
"#;

#[test]
fn claude_settings_powers_are_found() {
    let file = ".claude/settings.json";
    let map = scan_files(&[
        (file, CLAUDE_SETTINGS),
        ("src/settings.json", CLAUDE_SETTINGS),
    ]);

    assert_eq!(
        sightings(&map),
        expected(&[
            &format!("agent {file} tool_allow:WebFetch {file}:3"),
            &format!("agent {file} skip_permissions:bypassPermissions {file}:5"),
            &format!("agent {file} skip_permissions:enableAllProjectMcpServers {file}:7"),
            &format!("agent {file} hook:PostToolUse: ./scripts/lint.sh {file}:13"),
        ])
    );
}

#[test]
fn mcp_servers_are_found_in_every_client_format() {
    let map = scan_files(&[
        (".mcp.json", MCP_SERVERS_WITH_COMMENTS),
        (
            ".vscode/mcp.json",
            r#"{"servers": {"github": {"command": "docker", "args": ["run", "ghcr.io/github/github-mcp-server"]}}}"#,
        ),
        (".codex/config.toml", CODEX_CONFIG),
    ]);

    assert_eq!(
        sightings(&map),
        expected(&[
            "agent .mcp.json mcp_server:fs-tools: npx -y @acme/fs-mcp .mcp.json:4",
            "agent .mcp.json mcp_server:docs: https://docs.example.com/mcp .mcp.json:5",
            "agent .vscode/mcp.json mcp_server:github: docker run ghcr.io/github/github-mcp-server .vscode/mcp.json:1",
            "agent .codex/config.toml skip_permissions:approval_policy=never .codex/config.toml:1",
            "agent .codex/config.toml mcp_server:search: uvx search-mcp .codex/config.toml:4",
        ])
    );
}

const ALLOW_RULES: &str = r#"{
  "permissions": {
    "allow": [
      "Bash(git add:*)",
      "Bash(git push:*)",
      "Bash(yarn run lint)",
      "Bash(yarn run:*)",
      "Bash(python3 -m pytest tests/ -v)",
      "Bash(python3:*)",
      "Bash(gem search:*)",
      "Bash(yarn run prettier:*)",
      "Bash(curl -fsSL https://example.com/install.sh | sh)",
      "Read(//Users/dev/project/**)",
      "WebFetch(domain:docs.example.com)",
      "WebFetch(domain:raw.githubusercontent.com)",
      "WebSearch",
      "mcp__github__create_pull_request"
    ]
  }
}
"#;

#[test]
fn only_allow_rules_that_run_any_code_or_read_open_content_are_powers() {
    let file = ".claude/settings.local.json";
    let map = scan_files(&[(file, ALLOW_RULES)]);

    assert_eq!(
        sightings(&map),
        expected(&[
            &format!("agent {file} tool_allow:Bash(git push:*) {file}:5"),
            &format!("agent {file} tool_allow:Bash(yarn run:*) {file}:7"),
            &format!("agent {file} tool_allow:Bash(python3:*) {file}:9"),
            &format!(
                "agent {file} tool_allow:Bash(curl -fsSL https://example.com/install.sh | sh) {file}:12"
            ),
            &format!(
                "agent {file} tool_allow:WebFetch(domain:raw.githubusercontent.com) {file}:15"
            ),
            &format!("agent {file} tool_allow:WebSearch {file}:16"),
            &format!("agent {file} tool_allow:mcp__github__create_pull_request {file}:17"),
        ])
    );
}
