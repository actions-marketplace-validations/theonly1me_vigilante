use std::collections::BTreeMap;

use super::classify::{ScopeRoots, code_scope};
use super::graph::{Function, FunctionId, Graph};
use super::search::{Search, explore};
use super::{OwnPackagePath, note_kind_for_own_package};
use crate::Sighting;
use crate::repository_path::top_folder;

pub struct FolderExamples<'a> {
    pub graph: &'a Graph,
    pub scope: &'a ScopeRoots,
    pub allowed: &'a dyn Fn(&Function) -> bool,
}

pub fn folder_examples(input: FolderExamples) -> Vec<Sighting> {
    let FolderExamples {
        graph,
        scope,
        allowed,
    } = input;
    let mut folders: BTreeMap<&str, Vec<FunctionId>> = BTreeMap::new();
    for root in &scope.roots {
        folders
            .entry(top_folder(&graph.function(*root).path))
            .or_default()
            .push(*root);
    }
    if folders.len() < 2 {
        return Vec::new();
    }
    let mut examples = Vec::new();
    for roots in folders.into_values() {
        let explored = explore(Search {
            graph,
            roots: &roots,
            allowed,
        });
        for (power, route) in &explored.routes {
            let kind = note_kind_for_own_package(OwnPackagePath {
                capability: power.capability,
                stays_in_repository: true,
            });
            if kind.is_none() {
                examples.push(Sighting {
                    scope: code_scope(&scope.scope),
                    power: power.clone(),
                    evidence: graph.evidence_of(route),
                });
            }
        }
    }
    examples
}
