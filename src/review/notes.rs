use std::collections::BTreeSet;

use super::Note;
use crate::config::Config;
use crate::go::Reach;

pub struct NotesInput<'a> {
    pub base: &'a [Reach],
    pub head: Vec<Reach>,
    pub config: &'a Config,
}

pub fn new_notes(input: NotesInput) -> Vec<Note> {
    let known: BTreeSet<_> = input
        .base
        .iter()
        .map(|reach| (reach.through.as_str(), &reach.power))
        .collect();
    let mut noted = BTreeSet::new();
    let mut notes: Vec<Note> = input
        .head
        .into_iter()
        .filter(|reach| !known.contains(&(reach.through.as_str(), &reach.power)))
        .filter(|reach| noted.insert((reach.through.clone(), reach.power.clone())))
        .map(|reach| Note {
            hidden: input.config.is_hidden(reach.power.capability),
            scope: reach.scope,
            power: reach.power,
            through: reach.through,
            evidence: reach.evidence,
        })
        .collect();
    notes.sort_by(|left, right| {
        (&left.scope, &left.power, &left.through).cmp(&(&right.scope, &right.power, &right.through))
    });
    notes
}
