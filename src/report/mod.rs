mod examples;
mod json;
mod markdown;
mod notes;
mod result;
mod risks;
mod sarif;
mod test_code;
mod text;

pub use text::{scan_json, scan_text};

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
        Capability::ReadSystemState => "Reads environment variables or system information",
        Capability::Files => "Can read or change files",
        Capability::ModifySystemState => {
            "Can change environment variables, the working folder, or signal handlers"
        }
        Capability::OperatingSystem => "Uses other operating system functions",
        Capability::SystemCalls => "Can make system calls directly",
        Capability::ArbitraryExecution => "Can run code that the analysis cannot follow",
        Capability::Runtime => "Can change the language runtime",
        Capability::Unanalyzed => "Has calls that the analysis could not follow",
        Capability::TokenWrite => "The CI token can write to the repository",
        Capability::Oidc => "The CI job can get cloud credentials (OIDC)",
        Capability::UntrustedTrigger => {
            "The workflow runs with secrets on events that forks can start"
        }
        Capability::ThirdPartyAction => "The workflow runs a third-party action",
        Capability::Secret => "The workflow reads a secret",
        Capability::RemoteScript => "CI runs a script that it downloads",
        Capability::ToolAllow => "The agent can use a tool without asking",
        Capability::Hook => "The agent runs a command",
        Capability::McpServer => "The agent connects to an MCP server",
        Capability::SkipPermissions => "The agent skips permission checks",
        Capability::InstallHook => "Runs a command when dependencies install",
        Capability::GitHook => "Runs a command on git events",
    }
}
