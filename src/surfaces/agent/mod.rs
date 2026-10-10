mod claude;
mod codex;
mod mcp;

use crate::Detection;
use crate::document::Node;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentFile {
    ClaudeSettings,
    McpServers { key: &'static str },
    CodexConfig,
}

pub fn kind_of(path: &str) -> Option<AgentFile> {
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

pub fn detect_json(file: JsonFile) -> Vec<Detection> {
    match file.kind {
        AgentFile::ClaudeSettings => claude::detect(file.document),
        AgentFile::McpServers { key } => {
            file.document.get(key).map(mcp::detect).unwrap_or_default()
        }
        AgentFile::CodexConfig => Vec::new(),
    }
}

pub use codex::detect as detect_codex;

pub struct JsonFile<'a> {
    pub kind: AgentFile,
    pub document: &'a Node,
}

fn detection(found: Found) -> Detection {
    Detection {
        power: found.power,
        line: found.line,
        rule: "agent-config".to_string(),
    }
}

struct Found {
    power: crate::Power,
    line: usize,
}
