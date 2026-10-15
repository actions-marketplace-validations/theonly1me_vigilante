mod feeds;
mod install;
mod own_types;
mod parallel;
mod projects;
mod resolution;

use std::collections::{BTreeSet, HashMap};

use super::metadata::{Assembly, skeleton};
use crate::Error;
use crate::deep::Skip;
use crate::deep::program::{FileKey, PackageFile, Root};
use crate::registry::{Registry, Sources};
use crate::repository_path::{Folder, parent_folder};
use crate::tree::Tree;

const PROJECT_SUFFIX: &str = ".csproj";

pub struct Package {
    pub id: String,
    pub project: usize,
    pub assemblies: Vec<Assembly>,
    pub files: HashMap<String, (usize, usize)>,
}

pub struct EnvironmentInput<'a> {
    pub tree: &'a dyn Tree,
    pub registry: &'a Registry,
    pub sources: &'a Sources,
}

pub struct CSharpEnvironment<'a> {
    tree: &'a dyn Tree,
    registry: &'a Registry,
    sources: &'a Sources,
    own_types: HashMap<String, String>,
    project_of_file: HashMap<String, usize>,
    project_types: Vec<HashMap<String, (usize, String)>>,
    packages: Vec<Package>,
    package_of: HashMap<(String, String), Option<usize>>,
    skipped: BTreeSet<Skip>,
}

struct Search<'a> {
    start: &'a str,
    names: &'a [&'a str],
}

pub struct TypeQuery<'a> {
    pub project: Option<usize>,
    pub key: &'a str,
}

impl<'a> CSharpEnvironment<'a> {
    pub fn new(input: EnvironmentInput<'a>) -> Result<Self, Error> {
        Ok(Self {
            own_types: own_types::own_types(input.tree)?,
            tree: input.tree,
            registry: input.registry,
            sources: input.sources,
            project_of_file: HashMap::new(),
            project_types: Vec::new(),
            packages: Vec::new(),
            package_of: HashMap::new(),
            skipped: BTreeSet::new(),
        })
    }

    pub fn project_of(&mut self, path: &str) -> Result<Option<usize>, Error> {
        let mut folder = parent_folder(path).to_string();
        loop {
            let project = self
                .tree
                .paths()
                .iter()
                .find(|candidate| {
                    candidate.ends_with(PROJECT_SUFFIX) && parent_folder(candidate) == folder
                })
                .cloned();
            if let Some(project) = project {
                if let Some(known) = self.project_of_file.get(&project) {
                    return Ok(Some(*known));
                }
                return self.load(&project).map(Some);
            }
            if folder.is_empty() {
                return Ok(None);
            }
            folder = parent_folder(&folder).to_string();
        }
    }

    pub fn package_project(&self, package: usize) -> usize {
        self.packages[package].project
    }

    pub fn type_file(&self, query: TypeQuery) -> Option<FileKey> {
        if let Some(path) = self.own_types.get(query.key) {
            return Some(FileKey {
                root: Root::Repository,
                path: path.clone(),
            });
        }
        let (package, path) = self.project_types.get(query.project?)?.get(query.key)?;
        Some(FileKey {
            root: Root::Package(*package),
            path: path.clone(),
        })
    }

    pub fn read(&self, file: PackageFile) -> Option<String> {
        let package = self.packages.get(file.package)?;
        let (assembly, index) = package.files.get(file.path)?;
        Some(skeleton(&package.assemblies[*assembly].types[*index]))
    }

    pub fn package_name(&self, package: usize) -> String {
        self.packages[package].id.clone()
    }

    pub fn skip(&mut self, skip: Skip) {
        self.skipped.insert(skip);
    }

    pub fn into_skipped(self) -> Vec<Skip> {
        self.skipped.into_iter().collect()
    }

    fn nearest(&self, search: Search) -> Option<String> {
        let Search { start, names } = search;
        let mut folder = start.to_string();
        loop {
            for name in names {
                if let Some(path) = folder.joined(name)
                    && self.tree.paths().contains(&path)
                {
                    return Some(path);
                }
            }
            if folder.is_empty() {
                return None;
            }
            folder = parent_folder(&folder).to_string();
        }
    }
}
