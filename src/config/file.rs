use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Capability, Severity};

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigFile {
    pub scan: ScanSection,
    pub scopes: BTreeMap<String, String>,
    pub policy: PolicySection,
    pub report: ReportSection,
    pub severity: BTreeMap<Capability, Severity>,
    pub rules: RulesSection,
    pub ci: CiSection,
    pub engines: EnginesSection,
    pub registries: RegistriesSection,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ScanSection {
    pub exclude: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PolicySection {
    pub mode: Mode,
    pub deny: Vec<DenyEntry>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    Lock,
    Report,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DenyEntry {
    #[serde(default = "every_scope")]
    pub scope: String,
    pub capability: Capability,
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ReportSection {
    pub hide: Vec<Capability>,
}

impl Default for ReportSection {
    fn default() -> Self {
        Self {
            hide: vec![
                Capability::ReadSystemState,
                Capability::Files,
                Capability::OperatingSystem,
                Capability::Runtime,
                Capability::Unanalyzed,
            ],
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RulesSection {
    pub paths: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CiSection {
    pub trusted_owners: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EnginesSection {
    pub go: GoEngine,
    pub javascript: LanguageEngine,
    pub python: LanguageEngine,
    pub java: LanguageEngine,
    pub rust: LanguageEngine,
    pub csharp: LanguageEngine,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LanguageEngine {
    #[default]
    Deep,
    Patterns,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RegistriesSection {
    pub allow: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoEngine {
    #[default]
    Capslock,
    Patterns,
}

fn every_scope() -> String {
    "**".to_string()
}
