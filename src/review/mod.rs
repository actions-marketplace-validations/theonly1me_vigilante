mod notes;
mod run;

use std::path::PathBuf;

use serde::Serialize;

use crate::config::Mode;
use crate::{Error, Evidence, Power, Scope, Severity, Source};

#[derive(Clone, Debug)]
pub struct ReviewRequest {
    pub repository: PathBuf,
    pub base: Option<String>,
    pub head: Source,
}

#[derive(Clone, Debug, Serialize)]
pub struct Review {
    pub mode: ReviewMode,
    pub policy: Mode,
    pub findings: Vec<Finding>,
    pub notes: Vec<Note>,
    pub stale: Vec<LockEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReviewMode {
    Diff { base: String, head: String },
    Check,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub scope: Scope,
    pub power: Power,
    pub evidence: Evidence,
    pub severity: Severity,
    pub locked: bool,
    pub denied: bool,
    pub hidden: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Note {
    pub scope: Scope,
    pub power: Power,
    pub through: String,
    pub package: String,
    pub evidence: Evidence,
    pub hidden: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LockEntry {
    pub scope: Scope,
    pub power: Power,
}

pub fn review(request: ReviewRequest) -> Result<Review, Error> {
    match &request.base {
        Some(base) => run::diff(run::DiffRequest {
            repository: &request.repository,
            base,
            head: &request.head,
        }),
        None => run::check(run::CheckRequest {
            repository: &request.repository,
            head: &request.head,
        }),
    }
}

impl Review {
    pub fn passed(&self) -> bool {
        let lock_is_current = self.policy == Mode::Report || self.stale.is_empty();
        lock_is_current && !self.findings.iter().any(|finding| self.blocks(finding))
    }

    pub fn blocks(&self, finding: &Finding) -> bool {
        finding.denied || (self.policy == Mode::Lock && !finding.locked)
    }

    pub fn shown_findings(&self) -> impl Iterator<Item = &Finding> {
        self.findings
            .iter()
            .filter(|finding| !finding.hidden || self.blocks(finding))
    }

    pub fn hidden_count(&self) -> usize {
        self.findings.len() - self.shown_findings().count()
    }
}
