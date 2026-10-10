use std::collections::HashMap;

use super::paths::parent_folder;
use crate::Error;
use crate::lockfiles::{NPM_LOCKFILES, NpmLockfile};
use crate::tree::Tree;

pub struct Project {
    pub folder: String,
    pub file: String,
    pub lockfile: NpmLockfile,
}

pub struct Projects<'a> {
    tree: &'a dyn Tree,
    lockfile_of_folder: HashMap<String, String>,
    projects: Vec<Project>,
    loaded: HashMap<String, usize>,
}

impl<'a> Projects<'a> {
    pub fn new(tree: &'a dyn Tree) -> Self {
        let mut lockfile_of_folder = HashMap::new();
        for name in NPM_LOCKFILES.iter().rev() {
            for path in tree.paths() {
                if path == name || path.ends_with(&format!("/{name}")) {
                    lockfile_of_folder.insert(parent_folder(path).to_string(), path.clone());
                }
            }
        }
        Self {
            tree,
            lockfile_of_folder,
            projects: Vec::new(),
            loaded: HashMap::new(),
        }
    }

    pub fn project(&self, index: usize) -> &Project {
        &self.projects[index]
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

    pub fn read_repository(&self, path: &str) -> Result<Option<String>, Error> {
        Ok(self.tree.read(&[path])?.pop().map(|file| file.text))
    }

    pub fn has_repository_file(&self, path: &str) -> bool {
        self.tree.paths().contains(path)
    }

    fn load(&mut self, file: String) -> Result<usize, Error> {
        if let Some(index) = self.loaded.get(&file) {
            return Ok(*index);
        }
        let text = self.read_repository(&file)?.unwrap_or_default();
        let lockfile =
            NpmLockfile::parse(&text).map_err(|error| Error::new(format!("{file}: {error}")))?;
        self.projects.push(Project {
            folder: parent_folder(&file).to_string(),
            file: file.clone(),
            lockfile,
        });
        let index = self.projects.len() - 1;
        self.loaded.insert(file, index);
        Ok(index)
    }
}
