mod download;
mod install;

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

use super::bundler::{GemSource, LockedGem, read_lockfile};
use crate::Error;
use crate::deep::Skip;
use crate::deep::program::{FileKey, PackageFile, Root};
use crate::registry::{Prefetch, Registry, Sources};
use crate::repository_path::{Folder, parent_folder};
use crate::tree::Tree;

const LOCKFILES: &[&str] = &["Gemfile.lock", "gems.locked"];

pub struct InstalledGem {
    pub name: String,
    pub folder: PathBuf,
    pub files: BTreeSet<String>,
    pub lock: usize,
}

pub enum GemLocation {
    Own(String),
    Package(usize),
    Unreadable,
}

pub struct InstalledFile<'a> {
    pub lock: usize,
    pub path: &'a str,
}

pub struct GemQuery<'a> {
    pub lock: usize,
    pub name: &'a str,
}

pub struct EnvironmentInput<'a> {
    pub tree: &'a dyn Tree,
    pub registry: &'a Registry,
    pub sources: &'a Sources,
}

pub struct RubyEnvironment<'a> {
    tree: &'a dyn Tree,
    registry: &'a Registry,
    sources: &'a Sources,
    locks: Vec<Vec<LockedGem>>,
    lock_paths: Vec<String>,
    lock_of_path: HashMap<String, usize>,
    installed: Vec<InstalledGem>,
    install_of: HashMap<(usize, String), Option<usize>>,
    skipped: BTreeSet<Skip>,
}

impl<'a> RubyEnvironment<'a> {
    pub fn new(input: EnvironmentInput<'a>) -> Self {
        Self {
            tree: input.tree,
            registry: input.registry,
            sources: input.sources,
            locks: Vec::new(),
            lock_paths: Vec::new(),
            lock_of_path: HashMap::new(),
            installed: Vec::new(),
            install_of: HashMap::new(),
            skipped: BTreeSet::new(),
        }
    }

    pub fn lock_for(&mut self, path: &str) -> Result<Option<usize>, Error> {
        let mut folder = parent_folder(path).to_string();
        loop {
            for name in LOCKFILES {
                let candidate = folder.joined(name).unwrap_or_default();
                if !self.tree.paths().contains(&candidate) {
                    continue;
                }
                if let Some(known) = self.lock_of_path.get(&candidate) {
                    return Ok(Some(*known));
                }
                let text = self
                    .tree
                    .read(&[&candidate])?
                    .pop()
                    .map(|file| file.text)
                    .unwrap_or_default();
                self.locks.push(read_lockfile(&text));
                self.lock_paths.push(candidate.clone());
                self.lock_of_path.insert(candidate, self.locks.len() - 1);
                return Ok(Some(self.locks.len() - 1));
            }
            if folder.is_empty() {
                return Ok(None);
            }
            folder = parent_folder(&folder).to_string();
        }
    }

    pub fn gem(&mut self, query: GemQuery) -> Result<Option<GemLocation>, Error> {
        let GemQuery { lock, name } = query;
        let Some(gem) = self.locks[lock]
            .iter()
            .find(|gem| gem.name == name)
            .cloned()
        else {
            return Ok(None);
        };
        if let GemSource::Path { folder } = &gem.source {
            let base = parent_folder(&self.lock_paths[lock]);
            if let Some(own) = base.joined(folder)
                && (own.is_empty()
                    || self
                        .tree
                        .paths()
                        .iter()
                        .any(|path| path.starts_with(&format!("{own}/"))))
            {
                return Ok(Some(GemLocation::Own(own)));
            }
        }
        Ok(Some(
            self.install(install::Install { gem: &gem, lock })?
                .map_or(GemLocation::Unreadable, GemLocation::Package),
        ))
    }

    pub fn installed_with(&self, wanted: InstalledFile) -> Option<usize> {
        self.installed
            .iter()
            .position(|gem| gem.lock == wanted.lock && gem.files.contains(wanted.path))
    }

    pub fn locked_names(&self, lock: usize) -> Vec<String> {
        self.locks[lock]
            .iter()
            .map(|gem| gem.name.clone())
            .collect()
    }

    pub fn prefetch_lock(&self, lock: usize) {
        let planned: Vec<Prefetch> = self.locks[lock]
            .iter()
            .filter(|gem| !matches!(gem.source, GemSource::Path { .. }))
            .filter_map(|gem| match self.download_of(gem).ok()? {
                download::Planned::Download(download) => Some(Prefetch {
                    ecosystem: install::ECOSYSTEM,
                    name: gem.name.clone(),
                    version: gem.version.clone(),
                    download,
                    keep: install::is_kept,
                }),
                download::Planned::Skip(_) => None,
            })
            .collect();
        self.registry.prefetch(&planned);
    }

    pub fn installed(&self, package: usize) -> &InstalledGem {
        &self.installed[package]
    }

    pub fn exists(&self, file: &FileKey) -> bool {
        match file.root {
            Root::Repository => self.tree.paths().contains(&file.path),
            Root::Package(package) => self.installed[package].files.contains(&file.path),
        }
    }

    pub fn read(&self, file: PackageFile) -> Option<String> {
        let installed = self.installed.get(file.package)?;
        if !installed.files.contains(file.path) {
            return None;
        }
        std::fs::read_to_string(installed.folder.join(file.path)).ok()
    }

    pub fn skip(&mut self, skip: Skip) {
        self.skipped.insert(skip);
    }

    pub fn into_skipped(self) -> Vec<Skip> {
        self.skipped.into_iter().collect()
    }
}
