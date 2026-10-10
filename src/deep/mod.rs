mod classify;
pub mod graph;
mod search;

use serde::Serialize;

pub use classify::ScopeRoots;

use crate::{Capability, Evidence, Power, Scope, Sighting};

const BLOCKING_CAPABILITIES: [Capability; 3] =
    [Capability::Exec, Capability::Network, Capability::Native];

#[derive(Default)]
pub struct Analysis {
    pub sightings: Vec<Sighting>,
    pub reaches: Vec<Reach>,
    pub skipped: Vec<Skip>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Skip {
    pub source: String,
    pub reason: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReachKind {
    OnlyThroughOtherPackages,
    OwnCodeThatDoesNotBlock,
    NewPackageOnPath,
}

pub struct Reach {
    pub package: String,
    pub through: String,
    pub kind: ReachKind,
    pub power: Power,
    pub scope: Scope,
    pub evidence: Evidence,
}

pub struct OwnPackagePath {
    pub capability: Capability,
    pub stays_in_repository: bool,
}

pub fn note_kind_for_own_package(path: OwnPackagePath) -> Option<ReachKind> {
    if !path.stays_in_repository {
        Some(ReachKind::OnlyThroughOtherPackages)
    } else if !BLOCKING_CAPABILITIES.contains(&path.capability) {
        Some(ReachKind::OwnCodeThatDoesNotBlock)
    } else {
        None
    }
}
