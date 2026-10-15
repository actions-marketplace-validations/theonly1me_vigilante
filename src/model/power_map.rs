use std::collections::{BTreeMap, BTreeSet};

use super::{Evidence, Power, Scope};

pub type PowersByScope = BTreeMap<Scope, BTreeSet<Power>>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PowerMap {
    pub scopes: BTreeMap<Scope, BTreeMap<Power, BTreeSet<Evidence>>>,
}

#[derive(Clone, Debug)]
pub struct Sighting {
    pub scope: Scope,
    pub power: Power,
    pub evidence: Evidence,
}

impl PowerMap {
    pub fn insert(&mut self, sighting: Sighting) {
        self.scopes
            .entry(sighting.scope)
            .or_default()
            .entry(sighting.power)
            .or_default()
            .insert(sighting.evidence);
    }

    pub fn powers(&self) -> PowersByScope {
        self.scopes
            .iter()
            .map(|(scope, powers)| (scope.clone(), powers.keys().cloned().collect()))
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.scopes.is_empty()
    }
}
