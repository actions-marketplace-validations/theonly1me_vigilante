mod claude;
mod codex;
mod mcp;

use super::parse_json;
use crate::{Detection, Error, Power, SourceFile};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AgentFile {
    ClaudeSettings,
    McpServers { key: &'static str },
    CodexConfig,
}

pub fn handles(path: &str) -> bool {
    kind_of(path).is_some()
}

pub fn detect(file: SourceFile) -> Result<Vec<Detection>, Error> {
    match kind_of(file.path) {
        Some(AgentFile::ClaudeSettings) => Ok(claude::detect(&parse_json(file)?)),
        Some(AgentFile::McpServers { key }) => Ok(parse_json(file)?
            .get(key)
            .map(mcp::detect)
            .unwrap_or_default()),
        Some(AgentFile::CodexConfig) => codex::detect(file),
        None => Ok(Vec::new()),
    }
}

fn kind_of(path: &str) -> Option<AgentFile> {
    let mut parts = path.rsplit('/');
    let name = parts.next()?;
    let folder = parts.next().unwrap_or_default();
    match (folder, name) {
        (_, ".mcp.json") | (".cursor", "mcp.json") | (".gemini", "settings.json") => {
            Some(AgentFile::McpServers { key: "mcpServers" })
        }
        (".vscode", "mcp.json") => Some(AgentFile::McpServers { key: "servers" }),
        (".claude", "settings.json" | "settings.local.json") => Some(AgentFile::ClaudeSettings),
        (".codex", "config.toml") => Some(AgentFile::CodexConfig),
        _ => None,
    }
}

fn detection(found: Found) -> Detection {
    Detection {
        power: found.power,
        line: found.line,
        rule: "agent-config".to_string(),
    }
}

struct Found {
    power: Power,
    line: usize,
}
