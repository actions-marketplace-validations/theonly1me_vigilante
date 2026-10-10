mod crates;
mod download;
mod install;
mod lookup;

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

pub use crates::CrateRoot;
pub use lookup::{CrateId, DependencyLookup, Located};

use super::cargo::{LockedPackage, Manifest, read_manifest};
use crate::Error;
use crate::deep::Skip;
use crate::deep::program::{FileKey, PackageFile, Root};
use crate::registry::{Prefetch, Registry, Sources};
use crate::repository_path::parent_folder;
use crate::tree::Tree;

const MANIFEST: &str = "Cargo.toml";

pub struct OwnCrate {
    pub folder: String,
    pub manifest: Manifest,
}

pub struct Installed {
    pub name: String,
    pub version: String,
    pub folder: PathBuf,
    pub files: BTreeSet<String>,
    pub manifest: Manifest,
    pub lock: usize,
}

pub struct EnvironmentInput<'a> {
    pub tree: &'a dyn Tree,
    pub registry: &'a Registry,
    pub sources: &'a Sources,
}

pub struct RustEnvironment<'a> {
    tree: &'a dyn Tree,
    registry: &'a Registry,
    sources: &'a Sources,
    own: Vec<OwnCrate>,
    locks: Vec<Vec<LockedPackage>>,
    lock_paths: Vec<String>,
    lock_of_file: HashMap<String, usize>,
    installed: Vec<Installed>,
    install_of: HashMap<(String, String), Option<usize>>,
    skipped: BTreeSet<Skip>,
}

impl<'a> RustEnvironment<'a> {
    pub fn new(input: EnvironmentInput<'a>) -> Result<Self, Error> {
        let manifests: Vec<&str> = input
            .tree
            .paths()
            .iter()
            .map(String::as_str)
            .filter(|path| path.rsplit('/').next() == Some(MANIFEST))
            .collect();
        let read: Vec<OwnCrate> = input
            .tree
            .read(&manifests)?
            .into_iter()
            .filter_map(|file| {
                Some(OwnCrate {
                    folder: parent_folder(&file.path).to_string(),
                    manifest: read_manifest(&file.text)?,
                })
            })
            .collect();
        let own = read
            .iter()
            .filter(|own| own.manifest.name.is_some())
            .map(|own| OwnCrate {
                folder: own.folder.clone(),
                manifest: inherit(Inheritance {
                    member: own,
                    manifests: &read,
                }),
            })
            .collect();
        Ok(Self {
            tree: input.tree,
            registry: input.registry,
            sources: input.sources,
            own,
            locks: Vec::new(),
            lock_paths: Vec::new(),
            lock_of_file: HashMap::new(),
            installed: Vec::new(),
            install_of: HashMap::new(),
            skipped: BTreeSet::new(),
        })
    }

    pub fn exists(&self, file: &FileKey) -> bool {
        match file.root {
            Root::Repository => self.tree.paths().contains(&file.path),
            Root::Package(package) => self.installed[package].files.contains(&file.path),
        }
    }

    pub fn installed(&self, package: usize) -> &Installed {
        &self.installed[package]
    }

    pub fn read(&self, file: PackageFile) -> Option<String> {
        let installed = &self.installed[file.package];
        if !installed.files.contains(file.path) {
            return None;
        }
        std::fs::read_to_string(installed.folder.join(file.path)).ok()
    }

    pub fn prefetch(&self, planned: &[Prefetch]) {
        self.registry.prefetch(planned);
    }

    pub fn skip(&mut self, skip: Skip) {
        self.skipped.insert(skip);
    }

    pub fn into_skipped(self) -> Vec<Skip> {
        self.skipped.into_iter().collect()
    }
}

struct Inheritance<'a> {
    member: &'a OwnCrate,
    manifests: &'a [OwnCrate],
}

fn inherit(inheritance: Inheritance) -> Manifest {
    let Inheritance { member, manifests } = inheritance;
    let mut manifest = member.manifest.clone();
    let workspace = manifests
        .iter()
        .filter(|candidate| {
            !candidate.manifest.workspace_dependencies.is_empty()
                && (candidate.folder.is_empty()
                    || member.folder == candidate.folder
                    || member.folder.starts_with(&format!("{}/", candidate.folder)))
        })
        .max_by_key(|candidate| candidate.folder.len());
    if let Some(workspace) = workspace {
        for dependency in &mut manifest.dependencies {
            let is_inherited = manifest
                .inherited
                .iter()
                .any(|key| key.replace('-', "_") == dependency.local);
            if let Some(declared) = workspace
                .manifest
                .workspace_dependencies
                .iter()
                .find(|declared| is_inherited && declared.local == dependency.local)
            {
                dependency.package.clone_from(&declared.package);
            }
        }
    }
    manifest
}
