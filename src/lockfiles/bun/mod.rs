mod jsonc;
mod sources;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, HashMap};

use serde_json::Value;

use super::{Found, LockedPackage, Lookup, Place};
use crate::Error;
use crate::repository_path::Folder;
use sources::{Row, read_row};

#[derive(Debug, Default)]
pub struct BunLockfile {
    packages: HashMap<String, LockedPackage>,
    local_folders: BTreeMap<String, String>,
}

struct Nested<'a> {
    parent: &'a str,
    name: &'a str,
}

impl BunLockfile {
    pub fn parse(text: &str) -> Result<Self, Error> {
        let document: Value = serde_json::from_str(&jsonc::plain_json(text))
            .map_err(|error| Error::new(format!("invalid bun lockfile: {error}")))?;
        let mut lockfile = Self::default();
        let Some(packages) = document.get("packages").and_then(Value::as_object) else {
            return Ok(lockfile);
        };
        for (key, entry) in packages {
            let row = read_row(Row { key, entry })?;
            if let Some(local) = row.local_folder
                && !local.folder.is_empty()
                && local.folder != "."
            {
                lockfile
                    .local_folders
                    .entry(local.folder)
                    .or_insert(local.prefix);
            }
            lockfile.packages.insert(key.clone(), row.package);
        }
        Ok(lockfile)
    }

    pub fn find(&self, lookup: Lookup) -> Option<Found> {
        let name = lookup.name;
        let candidates = match lookup.place {
            Place::Repository { folder } => match self.owner_of(folder) {
                Some(parent) => nested_candidates(Nested { parent, name }),
                None => vec![name.to_string()],
            },
            Place::Package { key } => nested_candidates(Nested { parent: key, name }),
        };
        candidates.into_iter().find_map(|key| {
            let package = self.packages.get(&key)?.clone();
            Some(Found { key, package })
        })
    }

    fn owner_of(&self, folder: &str) -> Option<&str> {
        self.local_folders
            .iter()
            .filter(|(local, _)| local.holds(folder))
            .max_by_key(|(local, _)| local.len())
            .map(|(_, prefix)| prefix.as_str())
    }
}

fn nested_candidates(nested: Nested) -> Vec<String> {
    let parents = package_names(nested.parent);
    (0..=parents.len())
        .rev()
        .map(|length| {
            let mut parts = parents[..length].to_vec();
            parts.push(nested.name);
            parts.join("/")
        })
        .collect()
}

fn package_names(key: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let mut rest = key;
    while !rest.is_empty() {
        let skipped_slashes = usize::from(rest.starts_with('@'));
        match rest.match_indices('/').nth(skipped_slashes) {
            Some((end, _)) => {
                names.push(&rest[..end]);
                rest = &rest[end + 1..];
            }
            None => {
                names.push(rest);
                rest = "";
            }
        }
    }
    names
}
