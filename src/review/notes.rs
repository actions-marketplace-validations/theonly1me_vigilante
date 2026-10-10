use std::collections::BTreeSet;

use super::Note;
use crate::PowersByScope;
use crate::config::Config;
use crate::go::{Reach, ReachKind};

pub struct NotesInput<'a> {
    pub base: &'a [Reach],
    pub base_powers: &'a PowersByScope,
    pub head: Vec<Reach>,
    pub config: &'a Config,
}

pub fn new_notes(input: NotesInput) -> Vec<Note> {
    let known: BTreeSet<_> = input
        .base
        .iter()
        .map(|reach| (reach.package.as_str(), &reach.power))
        .collect();
    let had_power = |reach: &Reach| {
        reach.kind == ReachKind::OnlyThroughOtherPackages
            && input
                .base_powers
                .get(&reach.scope)
                .is_some_and(|powers| powers.contains(&reach.power))
    };
    let mut noted = BTreeSet::new();
    let mut notes: Vec<Note> = input
        .head
        .into_iter()
        .filter(|reach| !known.contains(&(reach.package.as_str(), &reach.power)))
        .filter(|reach| !had_power(reach))
        .filter(|reach| noted.insert((reach.package.clone(), reach.power.clone())))
        .map(|reach| Note {
            hidden: input.config.is_hidden(reach.power.capability),
            scope: reach.scope,
            power: reach.power,
            through: reach.through,
            package: reach.package,
            evidence: reach.evidence,
        })
        .collect();
    notes.sort_by(|left, right| {
        (&left.scope, &left.power, &left.through, &left.package).cmp(&(
            &right.scope,
            &right.power,
            &right.through,
            &right.package,
        ))
    });
    notes
}
