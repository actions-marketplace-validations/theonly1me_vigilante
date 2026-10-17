use std::collections::{BTreeSet, HashSet};

use crate::config::Config;
use crate::repository_path::parent_folder;
use crate::scan::Selection;
use crate::tree::Tree;

pub struct RootsInput<'a> {
    pub tree: &'a dyn Tree,
    pub config: &'a Config,
    pub scope_of: &'a dyn Fn(&str) -> String,
    pub selection: &'a Selection,
    pub is_code: fn(&str) -> bool,
    pub lockfiles: &'a [&'a str],
}

pub fn selected_roots(input: RootsInput) -> BTreeSet<String> {
    let changed_lockfile_folders: HashSet<&str> = match input.selection {
        Selection::Everything => HashSet::new(),
        Selection::Changes(changes) => changes
            .paths
            .iter()
            .filter(|path| {
                let name = path.rsplit('/').next().unwrap_or(path);
                input.lockfiles.contains(&name)
            })
            .map(|path| parent_folder(path))
            .collect(),
    };
    let is_selected = |path: &str| match input.selection {
        Selection::Everything => true,
        Selection::Changes(changes) => {
            changes.code_scopes.contains(&(input.scope_of)(path))
                || changed_lockfile_folders
                    .iter()
                    .any(|folder| folder.is_empty() || path.starts_with(&format!("{folder}/")))
        }
    };
    input
        .tree
        .paths()
        .iter()
        .filter(|path| {
            (input.is_code)(path) && !input.config.is_outside_analysis(path) && is_selected(path)
        })
        .cloned()
        .collect()
}
