use std::collections::{BTreeSet, HashSet};

use super::Program;
use super::link::{LinkInput, link};
use super::typed::TypedCalls;
use crate::deep::Analysis;

pub struct AnalyzeInput<'p> {
    pub roots: &'p BTreeSet<String>,
    pub types: Option<&'p TypedCalls>,
}

impl Program<'_> {
    pub fn analyze(&self, input: AnalyzeInput) -> Analysis {
        let by_name = analyzed(LinkInput {
            program: self,
            roots: input.roots,
            types: None,
        });
        let Some(types) = input.types.filter(|types| !types.is_empty()) else {
            return by_name;
        };
        let mut typed = analyzed(LinkInput {
            program: self,
            roots: input.roots,
            types: Some(types),
        });
        let found: HashSet<_> = typed
            .sightings
            .iter()
            .map(|sighting| (sighting.scope.clone(), sighting.power.clone()))
            .collect();
        typed.sightings.extend(
            by_name.sightings.into_iter().filter(|sighting| {
                !found.contains(&(sighting.scope.clone(), sighting.power.clone()))
            }),
        );
        typed
    }
}

fn analyzed(input: LinkInput) -> Analysis {
    let linked = link(input);
    linked.graph.analyze(&linked.scopes)
}
