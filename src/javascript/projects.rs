use std::collections::HashMap;

use super::manifest::Manifest;
use super::paths::parent_folder;
use crate::Error;
use crate::lockfiles::{JAVASCRIPT_LOCKFILES, JavaScriptLockfile, LockfileText, YARN_LOCKFILE};
use crate::repository_path::Folder;
use crate::tree::Tree;

pub struct Project {
    pub folder: String,
    pub file: String,
    pub lockfile: JavaScriptLockfile,
}

pub struct Dependency<'a> {
    pub path: &'a str,
    pub name: &'a str,
}

pub struct Projects<'a> {
    tree: &'a dyn Tree,
    lockfile_of_folder: HashMap<String, String>,
    projects: Vec<Project>,
    loaded: HashMap<String, usize>,
    manifests: HashMap<String, Manifest>,
}

impl<'a> Projects<'a> {
    pub fn new(tree: &'a dyn Tree) -> Self {
        let mut lockfile_of_folder = HashMap::new();
        for name in JAVASCRIPT_LOCKFILES.iter().rev() {
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
            manifests: HashMap::new(),
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

    pub fn declared(&mut self, dependency: Dependency) -> Result<Option<String>, Error> {
        let Dependency { path, name } = dependency;
        let mut folder = parent_folder(path);
        loop {
            let manifest_path = folder.joined("package.json").unwrap_or_default();
            if self.tree.paths().contains(&manifest_path) {
                if !self.manifests.contains_key(&manifest_path) {
                    let text = self.read_repository(&manifest_path)?.unwrap_or_default();
                    self.manifests
                        .insert(manifest_path.clone(), Manifest::parse(&text));
                }
                return Ok(self.manifests[&manifest_path].range_of(name));
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
        let mut lockfile = JavaScriptLockfile::parse(LockfileText {
            path: &file,
            text: &text,
        })
        .map_err(|error| Error::new(format!("{file}: {error}")))?;
        let folder = parent_folder(&file);
        if file.ends_with(YARN_LOCKFILE) {
            lockfile.add_workspaces(self.workspaces_under(folder)?);
        }
        self.projects.push(Project {
            folder: folder.to_string(),
            file: file.clone(),
            lockfile,
        });
        let index = self.projects.len() - 1;
        self.loaded.insert(file, index);
        Ok(index)
    }

    fn workspaces_under(&self, folder: &str) -> Result<HashMap<String, String>, Error> {
        let manifests: Vec<&str> = self
            .tree
            .paths()
            .iter()
            .map(String::as_str)
            .filter(|path| {
                path.rsplit('/').next() == Some("package.json")
                    && !path.contains("node_modules/")
                    && (folder.is_empty() || path.starts_with(&format!("{folder}/")))
            })
            .collect();
        let mut workspaces = HashMap::new();
        for manifest in self.tree.read(&manifests)? {
            if let Some(name) = Manifest::parse(&manifest.text).name() {
                let package_folder = parent_folder(&manifest.path);
                let relative = package_folder
                    .strip_prefix(folder)
                    .unwrap_or(package_folder)
                    .trim_start_matches('/');
                workspaces.insert(name.to_string(), relative.to_string());
            }
        }
        Ok(workspaces)
    }
}
