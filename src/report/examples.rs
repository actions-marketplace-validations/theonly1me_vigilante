use std::collections::BTreeSet;

use crate::Evidence;
use crate::repository_path::top_folder;

const MAXIMUM_FOLDERS: usize = 3;

pub struct Candidates<'a> {
    pub found: &'a BTreeSet<Evidence>,
    pub extra: Option<&'a BTreeSet<Evidence>>,
}

pub fn listed(candidates: Candidates) -> Vec<Evidence> {
    let mut listed: Vec<Evidence> = candidates.found.iter().cloned().collect();
    let mut folders: BTreeSet<&str> = candidates
        .found
        .iter()
        .map(|item| top_folder(&item.path))
        .collect();
    for item in candidates.extra.into_iter().flatten() {
        if folders.len() >= MAXIMUM_FOLDERS {
            break;
        }
        if folders.insert(top_folder(&item.path)) {
            listed.push(item.clone());
        }
    }
    listed
}

pub fn shown(candidates: Candidates) -> Vec<Evidence> {
    let mut folders = BTreeSet::new();
    listed(candidates)
        .into_iter()
        .filter(|item| folders.insert(top_folder(&item.path).to_string()))
        .take(MAXIMUM_FOLDERS)
        .collect()
}
