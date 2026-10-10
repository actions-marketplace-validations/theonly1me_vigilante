use std::collections::HashMap;

mod sources;

use super::{Found, LockedPackage, PackageSource, Place};
use sources::{name_and_version, source_of};
use crate::Error;
use crate::document::Node;
use crate::repository_path::joined;

const ROOT_IMPORTER: &str = ".";
const DEPENDENCY_LISTS: &[&str] = &["dependencies", "devDependencies", "optionalDependencies"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Slash,
    SlashAt,
    At,
}

#[derive(Debug)]
pub struct PnpmLockfile {
    format: Format,
    importers: HashMap<String, HashMap<String, String>>,
    resolutions: HashMap<String, PackageSource>,
    dependencies: HashMap<String, HashMap<String, String>>,
}

impl PnpmLockfile {
    pub fn parse(text: &str) -> Result<Self, Error> {
        let document = Node::parse(text)
            .map_err(|error| Error::new(format!("invalid pnpm lockfile: {error}")))?;
        let version = document
            .get("lockfileVersion")
            .and_then(Node::text)
            .unwrap_or_default();
        let major = version
            .trim_matches(['\'', '"'])
            .split('.')
            .next()
            .unwrap_or_default();
        let format = match major.parse::<u32>().unwrap_or(0) {
            9.. => Format::At,
            6..=8 => Format::SlashAt,
            _ => Format::Slash,
        };
        let mut importers = HashMap::new();
        match document.get("importers") {
            Some(listed) => {
                for entry in listed.entries() {
                    importers.insert(entry.key.clone(), references(&entry.value));
                }
            }
            None => {
                importers.insert(ROOT_IMPORTER.to_string(), references(&document));
            }
        }
        let mut resolutions = HashMap::new();
        let mut dependencies = HashMap::new();
        for entry in document
            .get("packages")
            .map(Node::entries)
            .unwrap_or_default()
        {
            let key = entry.key.trim_matches(['\'', '"']).to_string();
            resolutions.insert(key.clone(), source_of(&key, format, &entry.value));
            dependencies.insert(key, references(&entry.value));
        }
        for entry in document
            .get("snapshots")
            .map(Node::entries)
            .unwrap_or_default()
        {
            let key = entry.key.trim_matches(['\'', '"']).to_string();
            dependencies.insert(key, references(&entry.value));
        }
        Ok(Self {
            format,
            importers,
            resolutions,
            dependencies,
        })
    }

    pub fn find(&self, place: Place, name: &str) -> Option<Found> {
        let (reference, importer) = match place {
            Place::Repository { folder } => {
                let mut enclosing: Vec<(&String, &HashMap<String, String>)> = self
                    .importers
                    .iter()
                    .filter(|(importer, _)| contains(importer, folder))
                    .collect();
                enclosing.sort_by_key(|(importer, _)| std::cmp::Reverse(importer.len()));
                enclosing.into_iter().find_map(|(importer, references)| {
                    Some((references.get(name)?, importer.as_str()))
                })?
            }
            Place::Package { key } => (self.dependencies.get(key)?.get(name)?, ROOT_IMPORTER),
        };
        if let Some(link) = reference
            .strip_prefix("link:")
            .or_else(|| reference.strip_prefix("file:"))
        {
            let folder = joined(importer, link)?;
            return Some(Found {
                key: format!("link:{folder}"),
                package: LockedPackage {
                    name: name.to_string(),
                    version: reference.clone(),
                    source: PackageSource::Folder(folder),
                },
            });
        }
        let key = self.key_of(name, reference);
        let (package_name, version) = name_and_version(&key, self.format)?;
        let resolution_key = match self.format {
            Format::At => format!("{package_name}@{version}"),
            Format::Slash | Format::SlashAt => key.clone(),
        };
        let source = self.resolutions.get(&resolution_key)?.clone();
        Some(Found {
            key,
            package: LockedPackage {
                name: package_name,
                version,
                source,
            },
        })
    }

    fn key_of(&self, name: &str, reference: &str) -> String {
        let plain = reference.split('(').next().unwrap_or(reference);
        let is_alias = plain.strip_prefix('@').unwrap_or(plain).contains('@');
        match self.format {
            Format::At if is_alias => reference.to_string(),
            Format::At => format!("{name}@{reference}"),
            _ if reference.starts_with('/') => reference.to_string(),
            Format::SlashAt => format!("/{name}@{reference}"),
            Format::Slash => format!("/{name}/{reference}"),
        }
    }
}

fn references(node: &Node) -> HashMap<String, String> {
    let mut found = HashMap::new();
    for list in DEPENDENCY_LISTS {
        for entry in node.get(list).map(Node::entries).unwrap_or_default() {
            let version = entry
                .value
                .get("version")
                .and_then(Node::text)
                .or_else(|| entry.value.text());
            if let Some(version) = version {
                found.insert(entry.key.clone(), version.to_string());
            }
        }
    }
    found
}

fn contains(importer: &str, folder: &str) -> bool {
    importer == ROOT_IMPORTER
        || folder == importer
        || folder
            .strip_prefix(importer)
            .is_some_and(|rest| rest.starts_with('/'))
}
