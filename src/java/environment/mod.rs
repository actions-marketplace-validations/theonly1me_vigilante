mod archives;
mod load;
mod source;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::PathBuf;

use super::classfile;
use crate::Error;
use crate::deep::Skip;
use crate::deep::program::PackageFile;
use crate::registry::{Registry, Sources};
use crate::repository_path::parent_folder;
use crate::tree::Tree;

const BUILD_FILES: &[&str] = &[
    "build.gradle.kts",
    "build.gradle",
    "gradle.lockfile",
    "pom.xml",
];

pub struct Archive {
    pub name: String,
    pub folder: PathBuf,
    pub files: BTreeSet<String>,
    pub project: usize,
}

struct Project {
    classes: HashMap<String, usize>,
}

pub struct EnvironmentInput<'a> {
    pub tree: &'a dyn Tree,
    pub registry: &'a Registry,
    pub sources: &'a Sources,
}

pub struct JavaEnvironment<'a> {
    tree: &'a dyn Tree,
    registry: &'a Registry,
    sources: &'a Sources,
    build_of_folder: HashMap<String, String>,
    local_modules: HashSet<(String, String)>,
    projects: Vec<Project>,
    loaded: HashMap<String, usize>,
    archives: Vec<Archive>,
    skipped: BTreeSet<Skip>,
}

pub struct ClassLookup<'a> {
    pub project: usize,
    pub class_path: &'a str,
}

impl<'a> JavaEnvironment<'a> {
    pub fn new(input: EnvironmentInput<'a>) -> Self {
        let mut build_of_folder = HashMap::new();
        for name in BUILD_FILES {
            for path in input.tree.paths() {
                if path.rsplit('/').next() == Some(name) {
                    build_of_folder.insert(parent_folder(path).to_string(), path.clone());
                }
            }
        }
        Self {
            tree: input.tree,
            registry: input.registry,
            sources: input.sources,
            build_of_folder,
            local_modules: HashSet::new(),
            projects: Vec::new(),
            loaded: HashMap::new(),
            archives: Vec::new(),
            skipped: BTreeSet::new(),
        }
    }

    pub fn project_of(&mut self, path: &str) -> Result<Option<usize>, Error> {
        let mut folder = parent_folder(path);
        loop {
            if let Some(build) = self.build_of_folder.get(folder).cloned() {
                return self.load(build).map(Some);
            }
            if folder.is_empty() {
                return Ok(None);
            }
            folder = parent_folder(folder);
        }
    }

    pub fn archive(&self, index: usize) -> &Archive {
        &self.archives[index]
    }

    pub fn class(&self, lookup: ClassLookup) -> Option<usize> {
        self.projects
            .get(lookup.project)?
            .classes
            .get(lookup.class_path)
            .copied()
    }

    pub fn read(&self, file: PackageFile) -> Option<String> {
        let archive = &self.archives[file.package];
        if !archive.files.contains(file.path) {
            return None;
        }
        let bytes = std::fs::read(archive.folder.join(file.path)).ok()?;
        classfile::read(&bytes).map(|class| classfile::skeleton(&class))
    }

    pub fn skip(&mut self, skip: Skip) {
        self.skipped.insert(skip);
    }

    pub fn into_skipped(self) -> Vec<Skip> {
        self.skipped.into_iter().collect()
    }
}
