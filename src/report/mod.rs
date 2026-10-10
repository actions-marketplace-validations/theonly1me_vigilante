mod json;
mod markdown;
mod sarif;
mod text;

pub use text::power_map_text;

use crate::{Capability, Review};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Markdown,
    Json,
    Sarif,
}

impl Review {
    pub fn render(&self, format: Format) -> String {
        match format {
            Format::Markdown => markdown::render(self),
            Format::Json => json::render(self),
            Format::Sarif => sarif::render(self),
        }
    }
}

pub fn summary(capability: Capability) -> &'static str {
    match capability {
        Capability::Exec => "Can run other programs",
        Capability::Network => "Can use the network",
        Capability::Native => "Can run unsafe or native code",
        Capability::DynamicCode => "Can run code that is built at run time",
        Capability::Env => "Reads environment variables",
        Capability::TokenWrite => "The CI token can write to the repository",
        Capability::Oidc => "The CI job can get cloud credentials (OIDC)",
        Capability::UntrustedTrigger => {
            "The workflow runs with secrets on events that forks can start"
        }
        Capability::ThirdPartyAction => "The workflow runs a third-party action",
        Capability::Secret => "The workflow reads a secret",
        Capability::ToolAllow => "The agent can use a tool without asking",
        Capability::Hook => "The agent runs a command",
        Capability::McpServer => "The agent connects to an MCP server",
        Capability::SkipPermissions => "The agent skips permission checks",
        Capability::InstallHook => "Runs a command when dependencies install",
        Capability::GitHook => "Runs a command on git events",
    }
}
