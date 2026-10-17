use std::collections::BTreeSet;

use serde::Serialize;

use crate::deep::{Risk, RiskClass};
use crate::lock::Lock;
use crate::{Evidence, Power, Scope};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RiskFinding {
    pub scope: Scope,
    pub identity: String,
    pub power: Power,
    pub class: RiskClass,
    pub blocks: bool,
    pub approved: bool,
    pub evidence: Evidence,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct StaleRisk {
    pub scope: Scope,
    pub identity: String,
}

pub struct RiskChange<'a> {
    pub base: &'a [Risk],
    pub head: Vec<Risk>,
    pub lock: &'a Lock,
}

pub fn changed_risks(change: RiskChange) -> Vec<RiskFinding> {
    let known: BTreeSet<(&Scope, &str)> = change
        .base
        .iter()
        .map(|risk| (&risk.scope, risk.identity.as_str()))
        .collect();
    change
        .head
        .into_iter()
        .filter(|risk| risk.class != RiskClass::Constant)
        .filter(|risk| !known.contains(&(&risk.scope, risk.identity.as_str())))
        .map(|risk| {
            finding_of(Approval {
                risk,
                lock: change.lock,
            })
        })
        .collect()
}

pub struct RiskCheck<'a> {
    pub head: Vec<Risk>,
    pub lock: &'a Lock,
}

pub fn checked_risks(check: RiskCheck) -> (Vec<RiskFinding>, Vec<StaleRisk>) {
    let blocking: Vec<Risk> = check.head.into_iter().filter(Risk::blocks).collect();
    let current: BTreeSet<(&Scope, &str)> = blocking
        .iter()
        .map(|risk| (&risk.scope, risk.identity.as_str()))
        .collect();
    let stale = check
        .lock
        .risks
        .iter()
        .flat_map(|(scope, identities)| identities.iter().map(move |identity| (scope, identity)))
        .filter(|(scope, identity)| !current.contains(&(*scope, identity.as_str())))
        .map(|(scope, identity)| StaleRisk {
            scope: scope.clone(),
            identity: identity.clone(),
        })
        .collect();
    let findings = blocking
        .into_iter()
        .map(|risk| {
            finding_of(Approval {
                risk,
                lock: check.lock,
            })
        })
        .collect();
    (findings, stale)
}

struct Approval<'a> {
    risk: Risk,
    lock: &'a Lock,
}

fn finding_of(approval: Approval) -> RiskFinding {
    let Approval { risk, lock } = approval;
    RiskFinding {
        blocks: risk.blocks(),
        approved: lock.approves(&risk),
        scope: risk.scope,
        identity: risk.identity,
        power: risk.power,
        class: risk.class,
        evidence: risk.evidence,
    }
}
