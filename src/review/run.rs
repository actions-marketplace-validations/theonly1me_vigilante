use std::collections::BTreeSet;
use std::path::Path;

use super::{Finding, LockEntry, Review, ReviewMode};
use crate::config::Config;
use crate::lock::{Granted, Lock};
use crate::scan::{Changes, Location, Scanner, ScannerInput, Selection};
use crate::tree::{CommitPair, Git};
use crate::{Error, PowerMap, Sighting, Source};

pub struct DiffRequest<'a> {
    pub repository: &'a Path,
    pub base: &'a str,
    pub head: &'a Source,
}

pub struct CheckRequest<'a> {
    pub repository: &'a Path,
    pub head: &'a Source,
}

pub fn diff(request: DiffRequest) -> Result<Review, Error> {
    let Source::Commit(head_reference) = request.head else {
        return Err(Error::new("a diff needs a head commit"));
    };
    let git = Git::open(request.repository)?;
    let head = git.resolve(head_reference)?;
    let base_tip = git.resolve(request.base)?;
    let base = git.merge_base(CommitPair {
        base: &base_tip,
        head: &head,
    })?;
    let base_tree = git.tree(&base)?;
    let head_tree = git.tree(&head)?;
    let config = Config::load(&base_tree)?;
    let lock = Lock::load(&head_tree)?;
    let base_scanner = Scanner::new(ScannerInput {
        tree: &base_tree,
        config: &config,
        location: Location::Commit {
            repository: request.repository,
            commit: &base,
        },
    })?;
    let head_scanner = Scanner::new(ScannerInput {
        tree: &head_tree,
        config: &config,
        location: Location::Commit {
            repository: request.repository,
            commit: &head,
        },
    })?;
    let changed = git.changed_paths(CommitPair {
        base: &base,
        head: &head,
    })?;
    let affected_scopes: BTreeSet<String> = changed
        .iter()
        .flat_map(|path| {
            [
                base_scanner.code_scope_of(path),
                head_scanner.code_scope_of(path),
            ]
        })
        .collect();
    let selection = Selection::Changes(Changes {
        code_scopes: affected_scopes,
        paths: changed,
    });
    let base_powers = base_scanner.scan(&selection)?.powers();
    let head_map = head_scanner.scan(&selection)?;
    let judge = Judge {
        config: &config,
        lock: &lock,
    };
    let findings = judge.findings(
        sightings(&head_map)
            .filter(|sighting| {
                !base_powers
                    .get(&sighting.scope)
                    .is_some_and(|powers| powers.contains(&sighting.power))
            })
            .collect(),
    );
    Ok(Review {
        mode: ReviewMode::Diff { base, head },
        policy: config.mode,
        findings,
        stale: Vec::new(),
    })
}

pub fn check(request: CheckRequest) -> Result<Review, Error> {
    let located = request.head.locate(request.repository)?;
    let tree = located.tree.as_ref();
    let config = Config::load(tree)?;
    let lock = Lock::load(tree)?;
    let map = Scanner::new(ScannerInput {
        tree,
        config: &config,
        location: located.location(),
    })?
    .scan(&Selection::Everything)?;
    let judge = Judge {
        config: &config,
        lock: &lock,
    };
    let findings = judge
        .findings(sightings(&map).collect())
        .into_iter()
        .filter(|finding| !finding.locked || finding.denied)
        .collect();
    let current = map.powers();
    let stale = lock
        .powers
        .iter()
        .flat_map(|(scope, powers)| powers.iter().map(move |power| (scope, power)))
        .filter(|(scope, power)| {
            !current
                .get(*scope)
                .is_some_and(|found| found.contains(*power))
        })
        .map(|(scope, power)| LockEntry {
            scope: scope.clone(),
            power: power.clone(),
        })
        .collect();
    Ok(Review {
        mode: ReviewMode::Check,
        policy: config.mode,
        findings,
        stale,
    })
}

fn sightings(map: &PowerMap) -> impl Iterator<Item = Sighting> + '_ {
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

struct Judge<'a> {
    config: &'a Config,
    lock: &'a Lock,
}

impl Judge<'_> {
    fn findings(&self, sightings: Vec<Sighting>) -> Vec<Finding> {
        let mut findings: Vec<Finding> = sightings
            .into_iter()
            .map(|sighting| {
                let capability = sighting.power.capability;
                Finding {
                    locked: self.lock.contains(Granted {
                        scope: &sighting.scope,
                        power: &sighting.power,
                    }),
                    denied: self
                        .config
                        .denied_capabilities(&sighting.scope)
                        .contains(&capability),
                    hidden: self.config.is_hidden(capability),
                    severity: self.config.severity_of(capability),
                    scope: sighting.scope,
                    power: sighting.power,
                    evidence: sighting.evidence,
                }
            })
            .collect();
        findings.sort_by(|left, right| {
            right
                .severity
                .cmp(&left.severity)
                .then_with(|| left.scope.cmp(&right.scope))
                .then_with(|| left.power.cmp(&right.power))
        });
        findings
    }
}
