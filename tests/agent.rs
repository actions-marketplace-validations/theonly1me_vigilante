mod support;

use support::{expected, scan_files, sightings};

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
            &format!("agent {file} tool_allow:Bash(npm run test:*) {file}:3"),
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
