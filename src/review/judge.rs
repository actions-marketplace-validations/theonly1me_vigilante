use super::{Finding, Note};
use crate::config::Config;
use crate::deep::Reach;
use crate::lock::{Granted, Lock};
use crate::{Evidence, Power, PowerMap, Scope, Sighting};

pub struct Judge<'a> {
    pub config: &'a Config,
    pub lock: &'a Lock,
}

pub struct Subject<'a> {
    pub scope: &'a Scope,
    pub power: &'a Power,
}

pub struct Fact {
    pub scope: Scope,
    pub power: Power,
    pub evidence: Evidence,
}

impl Judge<'_> {
    pub fn denies(&self, subject: Subject) -> bool {
        self.config
            .denied_capabilities(subject.scope)
            .contains(&subject.power.capability)
    }

    pub fn finding(&self, fact: Fact) -> Finding {
        let capability = fact.power.capability;
        Finding {
            locked: self.lock.contains(Granted {
                scope: &fact.scope,
                power: &fact.power,
            }),
            denied: self.denies(Subject {
                scope: &fact.scope,
                power: &fact.power,
            }),
            hidden: self.config.is_hidden(capability),
            severity: self.config.severity_of(capability),
            scope: fact.scope,
            power: fact.power,
            evidence: fact.evidence,
        }
    }

    pub fn findings(&self, facts: Vec<Fact>) -> Vec<Finding> {
        let mut findings: Vec<Finding> = facts.into_iter().map(|fact| self.finding(fact)).collect();
        findings.sort_by(|left, right| {
            right
                .severity
                .cmp(&left.severity)
                .then_with(|| left.scope.cmp(&right.scope))
                .then_with(|| left.power.cmp(&right.power))
        });
        findings.dedup_by(|left, right| left.scope == right.scope && left.power == right.power);
        findings
    }

    pub fn test_findings(&self, facts: Vec<Fact>) -> Vec<Finding> {
        let mut findings: Vec<Finding> = facts.into_iter().map(|fact| self.finding(fact)).collect();
        findings.sort_by(|left, right| {
            (&left.scope, &left.evidence.path, &left.power).cmp(&(
                &right.scope,
                &right.evidence.path,
                &right.power,
            ))
        });
        findings.dedup_by(|left, right| {
            left.scope == right.scope
                && left.power == right.power
                && left.evidence.path == right.evidence.path
        });
        findings
    }

    pub fn split_denied(&self, notes: Vec<Note>) -> (Vec<Fact>, Vec<Note>) {
        let (denied, allowed): (Vec<Note>, Vec<Note>) = notes.into_iter().partition(|note| {
            self.denies(Subject {
                scope: &note.scope,
                power: &note.power,
            })
        });
        let facts = denied
            .into_iter()
            .map(|note| Fact {
                scope: note.scope,
                power: note.power,
                evidence: note.evidence,
            })
            .collect();
        (facts, allowed)
    }

    pub fn denied_reaches(&self, reaches: Vec<Reach>) -> Vec<Fact> {
        reaches
            .into_iter()
            .filter(|reach| {
                self.denies(Subject {
                    scope: &reach.scope,
                    power: &reach.power,
                })
            })
            .map(|reach| Fact {
                scope: reach.scope,
                power: reach.power,
                evidence: reach.evidence,
            })
            .collect()
    }
}

pub fn sightings(map: &PowerMap) -> impl Iterator<Item = Sighting> + '_ {
    map.scopes.iter().flat_map(|(scope, powers)| {
        powers.iter().filter_map(move |(power, evidence)| {
            Some(Sighting {
                scope: scope.clone(),
                power: power.clone(),
                evidence: evidence.first()?.clone(),
            })
        })
    })
}

pub fn facts_of(sightings: impl Iterator<Item = Sighting>) -> Vec<Fact> {
    sightings
        .map(|sighting| Fact {
            scope: sighting.scope,
            power: sighting.power,
            evidence: sighting.evidence,
        })
        .collect()
}
