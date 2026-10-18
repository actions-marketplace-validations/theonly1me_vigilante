mod classify;
mod examples;
pub mod graph;
pub mod program;
mod risks;
mod search;

use serde::Serialize;

pub use classify::ScopeRoots;
pub use risks::{RisksInput, risks};

use crate::{Capability, Evidence, Power, Scope, Sighting};

const BLOCKING_CAPABILITIES: [Capability; 3] =
    [Capability::Exec, Capability::Network, Capability::Native];

#[derive(Default)]
pub struct Analysis {
    pub sightings: Vec<Sighting>,
    pub examples: Vec<Sighting>,
    pub reaches: Vec<Reach>,
    pub risks: Vec<Risk>,
    pub skipped: Vec<Skip>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Risk {
    pub scope: Scope,
    pub identity: String,
    pub power: Power,
    pub class: RiskClass,
    pub evidence: Evidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskClass {
    Constant,
    Dynamic,
    Shell,
    Untrusted,
    Unpinned,
}

impl Risk {
    pub fn blocks(&self) -> bool {
        let capability = self.power.capability;
        match self.class {
            RiskClass::Untrusted => [
                Capability::Exec,
                Capability::DynamicCode,
                Capability::Network,
            ]
            .contains(&capability),
            RiskClass::Shell => capability == Capability::Exec,
            RiskClass::Constant | RiskClass::Dynamic | RiskClass::Unpinned => false,
        }
    }
}

impl Analysis {
    pub fn absorb(&mut self, other: Analysis) {
        self.sightings.extend(other.sightings);
        self.examples.extend(other.examples);
        self.reaches.extend(other.reaches);
        self.risks.extend(other.risks);
        self.skipped.extend(other.skipped);
    }
}

impl RiskClass {
    pub fn name(self) -> &'static str {
        match self {
            Self::Constant => "constant",
            Self::Dynamic => "dynamic",
            Self::Shell => "shell",
            Self::Untrusted => "untrusted",
            Self::Unpinned => "unpinned",
        }
    }
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
