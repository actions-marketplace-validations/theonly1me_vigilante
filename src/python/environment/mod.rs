mod install;
mod layout;
mod locate;
mod names;
mod plan;
#[cfg(test)]
mod tests;

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

use crate::Error;
use crate::deep::Skip;
use crate::deep::program::PackageFile;
use crate::lockfiles::python::{PYTHON_LOCKFILES, PythonLock, PythonLockText};
use crate::registry::{Registry, Sources};
use crate::repository_path::parent_folder;
use crate::tree::Tree;

pub use locate::{Located, Wanted};

pub struct Installed {
    pub name: String,
    pub folder: PathBuf,
    pub files: BTreeSet<String>,
    pub module_roots: Vec<String>,
    pub modules: BTreeSet<String>,
    pub project: usize,
}

pub struct Project {
    pub file: String,
    pub lock: PythonLock,
    by_module: HashMap<String, usize>,
    is_scanned: bool,
}

pub struct EnvironmentInput<'a> {
    pub tree: &'a dyn Tree,
    pub registry: &'a Registry,
    pub sources: &'a Sources,
}

pub struct Environment<'a> {
    tree: &'a dyn Tree,
    registry: &'a Registry,
    sources: &'a Sources,
    lockfile_of_folder: HashMap<String, String>,
    projects: Vec<Project>,
    loaded: HashMap<String, usize>,
    installed: Vec<Installed>,
    installed_at: HashMap<(usize, usize), Option<usize>>,
    skipped: BTreeSet<Skip>,
}

impl<'a> Environment<'a> {
    pub fn new(input: EnvironmentInput<'a>) -> Self {
        let mut lockfile_of_folder = HashMap::new();
        for name in PYTHON_LOCKFILES.iter().rev() {
            for path in input.tree.paths() {
                if path.rsplit('/').next() == Some(name) {
                    lockfile_of_folder.insert(parent_folder(path).to_string(), path.clone());
                }
            }
        }
        Self {
            tree: input.tree,
            registry: input.registry,
            sources: input.sources,
            lockfile_of_folder,
            projects: Vec::new(),
            loaded: HashMap::new(),
            installed: Vec::new(),
            installed_at: HashMap::new(),
            skipped: BTreeSet::new(),
        }
    }

    pub fn project_of(&mut self, path: &str) -> Result<Option<usize>, Error> {
        let mut folder = parent_folder(path);
        loop {
            if let Some(file) = self.lockfile_of_folder.get(folder).cloned() {
                return self.load(file).map(Some);
            }
            if folder.is_empty() {
                return Ok(None);
            }
            folder = parent_folder(folder);
        }
    }

    pub fn installed(&self, index: usize) -> &Installed {
        &self.installed[index]
    }

    pub fn read(&self, file: PackageFile) -> Option<String> {
        let installed = &self.installed[file.package];
        installed
            .files
            .contains(file.path)
            .then(|| std::fs::read_to_string(installed.folder.join(file.path)).ok())
            .flatten()
    }

    pub fn skip(&mut self, skip: Skip) {
        self.skipped.insert(skip);
    }

    pub fn into_skipped(self) -> Vec<Skip> {
        self.skipped.into_iter().collect()
    }

    fn load(&mut self, file: String) -> Result<usize, Error> {
        if let Some(index) = self.loaded.get(&file) {
            return Ok(*index);
        }
        let text = self
            .tree
            .read(&[&file])?
            .pop()
            .map(|read| read.text)
            .unwrap_or_default();
        let lock = PythonLock::parse(PythonLockText {
            path: &file,
            text: &text,
        })?;
        if !lock.unpinned.is_empty() {
            self.skip(Skip {
                source: file.clone(),
                reason: match lock.unpinned.as_slice() {
                    [only] => format!("`{only}` has no exact version, so Vigilante did not read it."),
                    many => format!(
                        "{} packages have no exact version (for example `{}`), so Vigilante did not read them.",
                        many.len(),
                        many[0]
                    ),
                },
            });
        }
        self.projects.push(Project {
            file: file.clone(),
            lock,
            by_module: HashMap::new(),
            is_scanned: false,
        });
        let index = self.projects.len() - 1;
        self.loaded.insert(file, index);
        Ok(index)
    }
}
