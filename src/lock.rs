use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::tree::Tree;
use crate::{Error, Power, PowerMap, PowersByScope, Scope, Surface};

const VERSION: u32 = 1;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Lock {
    pub powers: PowersByScope,
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
    powers: BTreeSet<Power>,
}

impl Lock {
    pub const FILE: &'static str = "vigilante.lock";

    pub fn of(map: &PowerMap) -> Self {
        Self {
            powers: map.powers(),
        }
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
        let powers = file
            .surfaces
            .into_iter()
            .flat_map(|(surface, scopes)| {
                scopes
                    .into_iter()
                    .map(move |(path, entry)| (Scope { surface, path }, entry.powers))
            })
            .collect();
        Ok(Self { powers })
    }

    pub fn render(&self) -> String {
        let mut surfaces: BTreeMap<Surface, BTreeMap<String, Entry>> = BTreeMap::new();
        for (scope, powers) in &self.powers {
            surfaces.entry(scope.surface).or_default().insert(
                scope.path.clone(),
                Entry {
                    powers: powers.clone(),
                },
            );
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
