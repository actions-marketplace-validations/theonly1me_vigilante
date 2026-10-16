use std::collections::BTreeSet;
use std::path::Path;

use super::judge::{Judge, facts_of, sightings};
use super::notes::{NotesInput, new_notes};
use super::{LockEntry, Review, ReviewMode};
use crate::config::Config;
use crate::lock::Lock;
use crate::scan::{Changes, Location, Scanner, ScannerInput, Selection};
use crate::tree::{CommitPair, Git};
use crate::{Error, Source};

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
        rules_tree: &base_tree,
        config: &config,
        location: Location::Commit {
            repository: request.repository,
            commit: &base,
        },
    })?;
    let head_scanner = Scanner::new(ScannerInput {
        tree: &head_tree,
        rules_tree: &base_tree,
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
    let base_scan = base_scanner.scan(&selection)?;
    let base_powers = base_scan.map.powers();
    let head_scan = head_scanner.scan(&selection)?;
    let judge = Judge {
        config: &config,
        lock: &lock,
    };
    let notes = new_notes(NotesInput {
        base: &base_scan.reaches,
        base_powers: &base_powers,
        head: head_scan.reaches,
        config: &config,
    });
    let (denied_notes, notes) = judge.split_denied(notes);
    let mut facts = facts_of(sightings(&head_scan.map).filter(|sighting| {
        !base_powers
            .get(&sighting.scope)
            .is_some_and(|powers| powers.contains(&sighting.power))
    }));
    facts.extend(denied_notes);
    let findings = judge.findings(facts);
    Ok(Review {
        mode: ReviewMode::Diff { base, head },
        policy: config.mode,
        findings,
        notes,
        stale: Vec::new(),
        new_gaps: head_scan
            .skipped
            .iter()
            .filter(|gap| !base_scan.skipped.contains(gap))
            .cloned()
            .collect(),
        skipped: head_scan.skipped,
        coverage: config.coverage,
    })
}

pub fn check(request: CheckRequest) -> Result<Review, Error> {
    let located = request.head.locate(request.repository)?;
    let tree = located.tree.as_ref();
    let config = Config::load(tree)?;
    let lock = Lock::load(tree)?;
    let scanned = Scanner::new(ScannerInput {
        tree,
        rules_tree: tree,
        config: &config,
        location: located.location(),
    })?
    .scan(&Selection::Everything)?;
    let map = scanned.map;
    let judge = Judge {
        config: &config,
        lock: &lock,
    };
    let mut facts = facts_of(sightings(&map));
    facts.extend(judge.denied_reaches(scanned.reaches));
    let findings = judge
        .findings(facts)
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
        notes: Vec::new(),
        stale,
        new_gaps: scanned.skipped.clone(),
        skipped: scanned.skipped,
        coverage: config.coverage,
    })
}
