use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::deep::Risk;
use crate::tree::Tree;
use crate::{Error, Power, PowerMap, PowersByScope, Scope, Surface};

const VERSION: u32 = 1;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Lock {
    pub powers: PowersByScope,
    pub risks: BTreeMap<Scope, BTreeSet<String>>,
}

#[derive(Serialize, Deserialize)]
struct LockFile {
    version: u32,
    #[serde(flatten)]
    surfaces: BTreeMap<Surface, BTreeMap<String, Entry>>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    #[serde(default)]
    powers: BTreeSet<Power>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    risks: BTreeSet<String>,
}

impl Lock {
    pub const FILE: &'static str = "vigilante.lock";

    pub fn of(map: &PowerMap) -> Self {
        Self {
            powers: map.powers(),
            risks: BTreeMap::new(),
        }
    }

    pub fn approving(mut self, risks: &[Risk]) -> Self {
        for risk in risks.iter().filter(|risk| risk.blocks()) {
            self.risks
                .entry(risk.scope.clone())
                .or_default()
                .insert(risk.identity.clone());
        }
        self
    }

    pub fn approves(&self, risk: &Risk) -> bool {
        self.risks
            .get(&risk.scope)
            .is_some_and(|identities| identities.contains(&risk.identity))
    }

    pub fn parse(text: &str) -> Result<Self, Error> {
        let file: LockFile = toml::from_str(text)
            .map_err(|error| Error::new(format!("invalid {}: {error}", Self::FILE)))?;
        if file.version != VERSION {
            return Err(Error::new(format!(
                "{} has version {}, and this vigilante reads version {VERSION}",
                Self::FILE,
                file.version
            )));
        }
        let mut powers = PowersByScope::new();
        let mut risks = BTreeMap::new();
        for (surface, scopes) in file.surfaces {
            for (path, entry) in scopes {
                let scope = Scope { surface, path };
                if !entry.risks.is_empty() {
                    risks.insert(scope.clone(), entry.risks);
                }
                powers.insert(scope, entry.powers);
            }
        }
        Ok(Self { powers, risks })
    }

    pub fn render(&self) -> String {
        let mut surfaces: BTreeMap<Surface, BTreeMap<String, Entry>> = BTreeMap::new();
        for (scope, powers) in &self.powers {
            surfaces.entry(scope.surface).or_default().insert(
                scope.path.clone(),
                Entry {
                    powers: powers.clone(),
                    risks: self.risks.get(scope).cloned().unwrap_or_default(),
                },
            );
        }
        for (scope, risks) in &self.risks {
            surfaces
                .entry(scope.surface)
                .or_default()
                .entry(scope.path.clone())
                .or_insert_with(|| Entry {
                    powers: BTreeSet::new(),
                    risks: risks.clone(),
                });
        }
        let file = LockFile {
            version: VERSION,
            surfaces,
        };
        toml::to_string_pretty(&file).unwrap_or_default()
    }

    pub fn contains(&self, granted: Granted) -> bool {
        self.powers
            .get(granted.scope)
            .is_some_and(|powers| powers.contains(granted.power))
    }

    pub(crate) fn load(tree: &dyn Tree) -> Result<Self, Error> {
        match tree.read(&[Self::FILE])?.pop() {
            Some(file) => Self::parse(&file.text),
            None => Ok(Self::default()),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Granted<'a> {
    pub scope: &'a Scope,
    pub power: &'a Power,
}
