use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Surface {
    Code,
    Ci,
    Agent,
    Build,
}

impl Surface {
    pub fn name(self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Ci => "ci",
            Self::Agent => "agent",
            Self::Build => "build",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Low,
    Medium,
    High,
}

impl Severity {
    pub fn name(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Exec,
    Network,
    Native,
    DynamicCode,
    Env,
    TokenWrite,
    Oidc,
    UntrustedTrigger,
    ThirdPartyAction,
    Secret,
    ToolAllow,
    Hook,
    McpServer,
    SkipPermissions,
    InstallHook,
    GitHook,
}

impl Capability {
    pub const ALL: [Capability; 16] = [
        Self::Exec,
        Self::Network,
        Self::Native,
        Self::DynamicCode,
        Self::Env,
        Self::TokenWrite,
        Self::Oidc,
        Self::UntrustedTrigger,
        Self::ThirdPartyAction,
        Self::Secret,
        Self::ToolAllow,
        Self::Hook,
        Self::McpServer,
        Self::SkipPermissions,
        Self::InstallHook,
        Self::GitHook,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Exec => "exec",
            Self::Network => "network",
            Self::Native => "native",
            Self::DynamicCode => "dynamic_code",
            Self::Env => "env",
            Self::TokenWrite => "token_write",
            Self::Oidc => "oidc",
            Self::UntrustedTrigger => "untrusted_trigger",
            Self::ThirdPartyAction => "third_party_action",
            Self::Secret => "secret",
            Self::ToolAllow => "tool_allow",
            Self::Hook => "hook",
            Self::McpServer => "mcp_server",
            Self::SkipPermissions => "skip_permissions",
            Self::InstallHook => "install_hook",
            Self::GitHook => "git_hook",
        }
    }

    pub fn surface(self) -> Surface {
        match self {
            Self::Exec | Self::Network | Self::Native | Self::DynamicCode | Self::Env => {
                Surface::Code
            }
            Self::TokenWrite
            | Self::Oidc
            | Self::UntrustedTrigger
            | Self::ThirdPartyAction
            | Self::Secret => Surface::Ci,
            Self::ToolAllow | Self::Hook | Self::McpServer | Self::SkipPermissions => {
                Surface::Agent
            }
            Self::InstallHook | Self::GitHook => Surface::Build,
        }
    }

    pub fn default_severity(self) -> Severity {
        match self {
            Self::Exec
            | Self::Native
            | Self::Oidc
            | Self::UntrustedTrigger
            | Self::Hook
            | Self::McpServer
            | Self::SkipPermissions
            | Self::InstallHook => Severity::High,
            Self::Network
            | Self::DynamicCode
            | Self::TokenWrite
            | Self::ThirdPartyAction
            | Self::Secret
            | Self::ToolAllow
            | Self::GitHook => Severity::Medium,
            Self::Env => Severity::Low,
        }
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

impl FromStr for Capability {
    type Err = Error;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|capability| capability.name() == name)
            .ok_or_else(|| Error::new(format!("unknown capability `{name}`")))
    }
}
