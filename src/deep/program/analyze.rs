use std::collections::{BTreeSet, HashSet};

use super::link::{LinkInput, link};
use super::typed::TypeCheck;
use super::{Program, Root};
use crate::deep::{Analysis, RisksInput, Skip, risks};

pub struct AnalyzeInput<'p> {
    pub roots: &'p BTreeSet<String>,
    pub check: Option<&'p TypeCheck>,
}

impl Program<'_> {
    pub fn analyze(&self, input: AnalyzeInput) -> Analysis {
        let mut analysis = self.analyze_calls(input);
        analysis.skipped.extend(self.load_gaps());
        analysis
    }

    fn load_gaps(&self) -> Vec<Skip> {
        self.files
            .iter()
            .filter(|file| file.key.root == Root::Repository)
            .flat_map(|file| {
                file.index.unresolved_loads.iter().map(|line| Skip {
                    source: file.key.path.clone(),
                    reason: format!(
                        "Line {line} loads a module by a name that Vigilante cannot read, so Vigilante did not follow it."
                    ),
                })
            })
            .collect()
    }

    fn analyze_calls(&self, input: AnalyzeInput) -> Analysis {
        let by_name = analyzed(LinkInput {
            program: self,
            roots: input.roots,
            types: None,
        });
        let types = match input.check {
            Some(TypeCheck::Checked(types)) if !types.is_empty() => types,
            Some(TypeCheck::NotChecked(reason)) => {
                let mut analysis = by_name;
                let scopes: BTreeSet<String> = input
                    .roots
                    .iter()
                    .map(|root| (self.input.scope_of)(root))
                    .collect();
                analysis
                    .skipped
                    .extend(scopes.into_iter().map(|source| Skip {
                        source,
                        reason: reason.clone(),
                    }));
                return analysis;
            }
            _ => return by_name,
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
        let found_examples: HashSet<_> = typed
            .examples
            .iter()
            .map(|sighting| (sighting.scope.clone(), sighting.power.clone()))
            .collect();
        typed
            .examples
            .extend(by_name.examples.into_iter().filter(|sighting| {
                !found_examples.contains(&(sighting.scope.clone(), sighting.power.clone()))
            }));
        let known: HashSet<_> = typed
            .risks
            .iter()
            .map(|risk| (risk.scope.clone(), risk.identity.clone()))
            .collect();
        typed.risks.extend(
            by_name
                .risks
                .into_iter()
                .filter(|risk| !known.contains(&(risk.scope.clone(), risk.identity.clone()))),
        );
        typed
    }
}

fn analyzed(input: LinkInput) -> Analysis {
    let linked = link(input);
    let mut analysis = linked.graph.analyze(&linked.scopes);
    analysis.risks = risks(RisksInput {
        graph: &linked.graph,
    });
    analysis
}
