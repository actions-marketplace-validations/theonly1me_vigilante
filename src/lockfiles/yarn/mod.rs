mod berry;
mod classic;
#[cfg(test)]
mod tests;

use std::collections::HashMap;

use super::{Found, LockedPackage, Lookup, PackageSource, Place};
use crate::Error;
use crate::repository_path::Folder;

#[derive(Debug, Default)]
pub struct YarnLockfile {
    packages: Vec<YarnPackage>,
    descriptors: HashMap<String, usize>,
    names: HashMap<String, Vec<usize>>,
    workspace_dependencies: HashMap<String, HashMap<String, String>>,
    workspace_folders: HashMap<String, String>,
}

struct WorkspaceDependency<'a> {
    folder: &'a str,
    name: &'a str,
}

#[derive(Debug)]
struct YarnPackage {
    descriptors: Vec<String>,
    key: String,
    package: LockedPackage,
    dependencies: HashMap<String, String>,
}

impl YarnLockfile {
    pub fn parse(text: &str) -> Result<Self, Error> {
        if text.lines().any(|line| line.starts_with("__metadata:")) {
            berry::parse(text)
        } else {
            Ok(classic::parse(text))
        }
    }

    pub fn add_workspaces(&mut self, workspaces: HashMap<String, String>) {
        for (name, folder) in workspaces {
            self.workspace_folders.entry(name).or_insert(folder);
        }
    }

    pub fn find(&self, lookup: Lookup) -> Option<Found> {
        let name = lookup.name;
        let descriptor = match lookup.place {
            Place::Package { key } => {
                let index = *self.descriptors.get(key)?;
                Some(self.packages[index].dependencies.get(name)?.clone())
            }
            Place::Repository { folder } => {
                if let Some(workspace) = self.workspace_folders.get(name) {
                    return Some(Found {
                        key: format!("{name}@workspace:{workspace}"),
                        package: LockedPackage {
                            name: name.to_string(),
                            version: String::new(),
                            source: PackageSource::Folder(workspace.clone()),
                        },
                    });
                }
                self.workspace_descriptor(WorkspaceDependency { folder, name })
                    .or_else(|| lookup.declared.map(|range| format!("{name}@{range}")))
            }
        };
        let index = descriptor
            .and_then(|descriptor| self.index_of(&descriptor))
            .or_else(|| self.only_version_of(name))?;
        let package = &self.packages[index];
        Some(Found {
            key: package.key.clone(),
            package: package.package.clone(),
        })
    }

    fn workspace_descriptor(&self, wanted: WorkspaceDependency) -> Option<String> {
        let WorkspaceDependency { folder, name } = wanted;
        let mut enclosing: Vec<(&String, &HashMap<String, String>)> = self
            .workspace_dependencies
            .iter()
            .filter(|(workspace, _)| workspace.holds(folder))
            .collect();
        enclosing.sort_by_key(|(workspace, _)| std::cmp::Reverse(workspace.len()));
        enclosing
            .into_iter()
            .find_map(|(_, dependencies)| dependencies.get(name).cloned())
    }

    fn index_of(&self, descriptor: &str) -> Option<usize> {
        self.descriptors.get(descriptor).copied().or_else(|| {
            let (name, range) = split_descriptor(descriptor)?;
            self.descriptors
                .get(&format!("{name}@npm:{range}"))
                .copied()
        })
    }

    fn only_version_of(&self, name: &str) -> Option<usize> {
        match self.names.get(name)?.as_slice() {
            [only] => Some(*only),
            _ => None,
        }
    }

    fn add(&mut self, package: YarnPackage) {
        let index = self.packages.len();
        for descriptor in package.descriptors.iter().chain([&package.key]) {
            let descriptor = descriptor.clone();
            self.descriptors.insert(descriptor, index);
        }
        self.names
            .entry(package.package.name.clone())
            .or_default()
            .push(index);
        self.packages.push(package);
    }
}

fn split_descriptor(descriptor: &str) -> Option<(&str, &str)> {
    let search_from = usize::from(descriptor.starts_with('@'));
    let at = descriptor[search_from..].find('@')? + search_from;
    Some((&descriptor[..at], &descriptor[at + 1..]))
}

fn unquoted(text: &str) -> &str {
    text.trim().trim_matches('"')
}
