mod decide;
mod locate;

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use super::manifest::Manifest;
use super::projects::Projects;
use crate::deep::Skip;
use crate::registry::{Registry, Sources};

pub struct Installed {
    pub name: String,
    pub folder: PathBuf,
    pub files: BTreeSet<String>,
    pub manifest: Manifest,
    pub project: usize,
    pub key: String,
}

pub enum Located {
    Repository(String),
    Package(usize),
    Missing,
}

pub struct Locate<'a> {
    pub project: usize,
    pub folder: &'a str,
    pub name: &'a str,
    pub is_from_repository: bool,
    pub declared: Option<&'a str>,
}

pub struct PackageFile<'a> {
    pub package: usize,
    pub path: &'a str,
}

pub struct PackagesInput<'a> {
    pub projects: Projects<'a>,
    pub registry: &'a Registry,
    pub sources: &'a Sources,
}

pub struct Packages<'a> {
    pub projects: Projects<'a>,
    registry: &'a Registry,
    sources: &'a Sources,
    installed: Vec<Installed>,
    installed_at: HashMap<(usize, String), Option<usize>>,
    skipped: BTreeSet<Skip>,
}

impl<'a> Packages<'a> {
    pub fn new(input: PackagesInput<'a>) -> Self {
        Self {
            projects: input.projects,
            registry: input.registry,
            sources: input.sources,
            installed: Vec::new(),
            installed_at: HashMap::new(),
            skipped: BTreeSet::new(),
        }
    }

    pub fn installed(&self, index: usize) -> &Installed {
        &self.installed[index]
    }

    pub fn names(&self) -> Vec<String> {
        self.installed
            .iter()
            .map(|installed| installed.name.clone())
            .collect()
    }

    pub fn skip(&mut self, skip: Skip) {
        self.skipped.insert(skip);
    }

    pub fn into_skipped(self) -> Vec<Skip> {
        self.skipped.into_iter().collect()
    }

    pub fn read(&self, file: PackageFile) -> Option<String> {
        let installed = &self.installed[file.package];
        installed
            .files
            .contains(file.path)
            .then(|| std::fs::read_to_string(Path::new(&installed.folder).join(file.path)).ok())
            .flatten()
    }
}
