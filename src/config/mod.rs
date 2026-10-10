mod excludes;
mod file;

use std::collections::{BTreeMap, BTreeSet};

use globset::{Glob, GlobBuilder, GlobMatcher, GlobSet, GlobSetBuilder};

pub use file::{GoEngine, LanguageEngine, Mode};

use crate::registry::Sources;
use crate::tree::Tree;
use crate::{Capability, Error, Power, Scope, Severity};
use excludes::TEST_AND_VENDORED_PATHS;
use file::ConfigFile;

pub struct Config {
    pub mode: Mode,
    pub rule_paths: Vec<String>,
    pub go_engine: GoEngine,
    pub javascript_engine: LanguageEngine,
    registry_sources: Sources,
    excluded: GlobSet,
    named_scopes: Vec<(String, GlobMatcher)>,
    denied: Vec<(GlobMatcher, Capability)>,
    hidden: BTreeSet<Capability>,
    severity: BTreeMap<Capability, Severity>,
    trusted_owners: Vec<String>,
}

impl Config {
    pub const FILE: &'static str = "vigilante.toml";

    pub fn load(tree: &dyn Tree) -> Result<Self, Error> {
        match tree.read(&[Self::FILE])?.pop() {
            Some(file) => Self::parse(&file.text),
            None => Self::parse(""),
        }
    }

    pub fn parse(text: &str) -> Result<Self, Error> {
        let file: ConfigFile = toml::from_str(text)
            .map_err(|error| Error::new(format!("invalid {}: {error}", Self::FILE)))?;
        let mut excluded = GlobSetBuilder::new();
        for pattern in TEST_AND_VENDORED_PATHS
            .iter()
            .copied()
            .chain(file.scan.exclude.iter().map(String::as_str))
        {
            excluded.add(glob(pattern)?);
        }
        let named_scopes = file
            .scopes
            .into_iter()
            .map(|(name, pattern)| Ok((name, glob(&pattern)?.compile_matcher())))
            .collect::<Result<_, Error>>()?;
        let denied = file
            .policy
            .deny
            .iter()
            .map(|entry| Ok((glob(&entry.scope)?.compile_matcher(), entry.capability)))
            .collect::<Result<_, Error>>()?;
        Ok(Self {
            mode: file.policy.mode,
            rule_paths: file.rules.paths,
            go_engine: file.engines.go,
            javascript_engine: file.engines.javascript,
            registry_sources: Sources::new(&file.registries.allow)?,
            excluded: excluded
                .build()
                .map_err(|error| Error::new(format!("invalid exclude pattern: {error}")))?,
            named_scopes,
            denied,
            hidden: file.report.hide.into_iter().collect(),
            severity: file.severity,
            trusted_owners: file.ci.trusted_owners,
        })
    }

    pub fn registry_sources(&self) -> &Sources {
        &self.registry_sources
    }

    pub fn is_excluded(&self, path: &str) -> bool {
        self.excluded.is_match(path)
    }

    pub fn named_scope_of(&self, path: &str) -> Option<&str> {
        self.named_scopes
            .iter()
            .find(|(_, matcher)| matcher.is_match(path))
            .map(|(name, _)| name.as_str())
    }

    pub fn denied_capabilities(&self, scope: &Scope) -> BTreeSet<Capability> {
        self.denied
            .iter()
            .filter(|(matcher, _)| matcher.is_match(&scope.path))
            .map(|(_, capability)| *capability)
            .collect()
    }

    pub fn is_hidden(&self, capability: Capability) -> bool {
        self.hidden.contains(&capability)
    }

    pub fn trusts(&self, power: &Power) -> bool {
        let owner = power
            .detail
            .as_deref()
            .and_then(|action| action.split_once('/'))
            .map(|(owner, _)| owner);
        power.capability == Capability::ThirdPartyAction
            && owner.is_some_and(|owner| {
                self.trusted_owners
                    .iter()
                    .any(|trusted| trusted.eq_ignore_ascii_case(owner))
            })
    }

    pub fn severity_of(&self, capability: Capability) -> Severity {
        self.severity
            .get(&capability)
            .copied()
            .unwrap_or_else(|| capability.default_severity())
    }
}

fn glob(pattern: &str) -> Result<Glob, Error> {
    GlobBuilder::new(pattern)
        .literal_separator(true)
        .build()
        .map_err(|error| Error::new(format!("invalid pattern `{pattern}`: {error}")))
}
