use std::collections::HashMap;

use super::packages::Packages;
use super::syntax::FileIndex;
use crate::code::Rules;
use crate::deep::graph::Owner;
use crate::tree::Tree;
use crate::{Capability, Error};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Root {
    Repository,
    Package(usize),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileKey {
    pub root: Root,
    pub path: String,
}

pub struct Privilege {
    pub capability: Capability,
    pub rule: String,
}

pub struct SiteSink {
    pub definition: usize,
    pub line: usize,
    pub name: String,
    pub privilege: Privilege,
}

pub struct LoadedFile {
    pub key: FileKey,
    pub display: String,
    pub owner: Owner,
    pub index: FileIndex,
    pub privileged: HashMap<String, Vec<Privilege>>,
    pub sites: Vec<SiteSink>,
    pub resolved: HashMap<String, FileKey>,
}

pub struct ProgramInput<'a> {
    pub tree: &'a dyn Tree,
    pub rules: &'a Rules,
    pub scope_of: &'a dyn Fn(&str) -> String,
    pub packages: Packages<'a>,
}

pub struct Program<'a> {
    pub(super) input: ProgramInput<'a>,
    pub files: Vec<LoadedFile>,
    pub ids: HashMap<FileKey, usize>,
}

impl<'a> Program<'a> {
    pub fn new(input: ProgramInput<'a>) -> Self {
        Self {
            input,
            files: Vec::new(),
            ids: HashMap::new(),
        }
    }

    pub fn packages(&self) -> &Packages<'a> {
        &self.input.packages
    }

    pub fn into_packages(self) -> Packages<'a> {
        self.input.packages
    }

    pub fn load(&mut self, roots: &[String]) -> Result<(), Error> {
        let mut frontier: Vec<FileKey> = roots
            .iter()
            .map(|path| FileKey {
                root: Root::Repository,
                path: path.clone(),
            })
            .collect();
        while !frontier.is_empty() {
            frontier.retain(|key| !self.ids.contains_key(key));
            frontier.sort();
            frontier.dedup();
            let repository_paths: Vec<&str> = frontier
                .iter()
                .filter(|key| key.root == Root::Repository)
                .map(|key| key.path.as_str())
                .collect();
            let mut texts: HashMap<String, String> = self
                .input
                .tree
                .read(&repository_paths)?
                .into_iter()
                .map(|file| (file.path, file.text))
                .collect();
            let mut next = Vec::new();
            for key in frontier {
                let text = match key.root {
                    Root::Repository => texts.remove(&key.path),
                    Root::Package(index) => self.input.packages.read(index, &key.path),
                };
                let Some(text) = text else {
                    continue;
                };
                let file = self.parse(key, &text)?;
                next.extend(file.resolved.values().cloned());
                self.ids.insert(file.key.clone(), self.files.len());
                self.files.push(file);
            }
            frontier = next;
        }
        Ok(())
    }
}
