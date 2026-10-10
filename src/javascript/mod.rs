mod files;
mod link;
mod manifest;
mod packages;
mod parse;
mod paths;
mod program;
mod projects;
mod resolve;
mod syntax;

use std::collections::{BTreeSet, HashSet};

use ast_grep_language::SupportLang;

use crate::Error;
use crate::code::{Rules, language_of};
use crate::config::Config;
use crate::deep::{self, Analysis};
use crate::lockfiles::NPM_LOCKFILES;
use crate::registry::Registry;
use crate::scan::Selection;
use crate::tree::Tree;
use packages::{Packages, PackagesInput};
use paths::{is_code_file, parent_folder};
use program::{Program, ProgramInput};
use projects::Projects;

pub struct JavaScriptRequest<'a> {
    pub tree: &'a dyn Tree,
    pub config: &'a Config,
    pub rules: &'a Rules,
    pub scope_of: &'a dyn Fn(&str) -> String,
    pub selection: &'a Selection,
}

pub fn owns(path: &str) -> bool {
    language_of(path) == Some(SupportLang::Tsx)
}

pub fn analyze(request: JavaScriptRequest) -> Result<Analysis, Error> {
    let changed_lockfile_folders: HashSet<&str> = match request.selection {
        Selection::Everything => HashSet::new(),
        Selection::Changes(changes) => changes
            .paths
            .iter()
            .filter(|path| {
                let name = path.rsplit('/').next().unwrap_or(path);
                NPM_LOCKFILES.contains(&name)
            })
            .map(|path| parent_folder(path))
            .collect(),
    };
    let is_selected = |path: &str| match request.selection {
        Selection::Everything => true,
        Selection::Changes(changes) => {
            changes.code_scopes.contains(&(request.scope_of)(path))
                || changed_lockfile_folders
                    .iter()
                    .any(|folder| folder.is_empty() || path.starts_with(&format!("{folder}/")))
        }
    };
    let roots: BTreeSet<String> = request
        .tree
        .paths()
        .iter()
        .filter(|path| is_code_file(path) && !request.config.is_excluded(path) && is_selected(path))
        .cloned()
        .collect();
    if roots.is_empty() {
        return Ok(Analysis::default());
    }
    let registry = Registry::new(Registry::default_cache_root());
    let mut program = Program::new(ProgramInput {
        tree: request.tree,
        rules: request.rules,
        scope_of: request.scope_of,
        packages: Packages::new(PackagesInput {
            projects: Projects::new(request.tree),
            registry: &registry,
            sources: request.config.registry_sources(),
        }),
    });
    let root_paths: Vec<String> = roots.iter().cloned().collect();
    program.load(&root_paths)?;
    let linked = link::link(&program, &roots);
    let mut analysis = deep::analyze(&linked.graph, &linked.scopes);
    analysis.skipped = program.into_packages().into_skipped();
    Ok(analysis)
}
