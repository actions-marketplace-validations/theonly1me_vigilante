mod excludes;
mod file;

use std::collections::{BTreeMap, BTreeSet};

use globset::{Glob, GlobBuilder, GlobMatcher, GlobSet, GlobSetBuilder};

pub use file::Mode;

use crate::tree::Tree;
use crate::{Capability, Error, Scope, Severity};
use excludes::TEST_AND_VENDORED_PATHS;
use file::ConfigFile;

pub struct Config {
    pub mode: Mode,
    pub rule_paths: Vec<String>,
    excluded: GlobSet,
    named_scopes: Vec<(String, GlobMatcher)>,
    denied: Vec<(GlobMatcher, Capability)>,
    hidden: BTreeSet<Capability>,
    severity: BTreeMap<Capability, Severity>,
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
            excluded: excluded
                .build()
                .map_err(|error| Error::new(format!("invalid exclude pattern: {error}")))?,
            named_scopes,
            denied,
            hidden: file.report.hide.into_iter().collect(),
            severity: file.severity,
        })
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
