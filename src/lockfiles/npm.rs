use std::collections::HashMap;

use serde_json::{Map, Value};

use super::{Found, LockedPackage, PackageSource, Place, default_url};
use crate::Error;

const INSTALL_FOLDER: &str = "node_modules/";

#[derive(Debug, Default)]
pub struct NpmLockfile {
    packages: HashMap<String, LockedPackage>,
}

impl NpmLockfile {
    pub fn parse(text: &str) -> Result<Self, Error> {
        let document: Value = serde_json::from_str(text)
            .map_err(|error| Error::new(format!("invalid npm lockfile: {error}")))?;
        let mut lockfile = Self::default();
        if let Some(packages) = document.get("packages").and_then(Value::as_object) {
            for (key, entry) in packages {
                if !key.is_empty() {
                    lockfile.insert(key, entry, false);
                }
            }
        } else if let Some(dependencies) = document.get("dependencies").and_then(Value::as_object) {
            lockfile.insert_tree("", dependencies);
        }
        Ok(lockfile)
    }

    pub fn find(&self, place: Place, name: &str) -> Option<Found> {
        let folder = match place {
            Place::Repository { folder } => folder,
            Place::Package { key } => key,
        };
        install_candidates(folder, name)
            .into_iter()
            .find_map(|key| {
                let package = self.packages.get(&key)?.clone();
                Some(Found { key, package })
            })
    }

    fn insert_tree(&mut self, parent: &str, dependencies: &Map<String, Value>) {
        for (name, entry) in dependencies {
            let key = format!("{parent}{INSTALL_FOLDER}{name}");
            self.insert(&key, entry, true);
            if let Some(nested) = entry.get("dependencies").and_then(Value::as_object) {
                self.insert_tree(&format!("{key}/"), nested);
            }
        }
    }

    fn insert(&mut self, key: &str, entry: &Value, is_version_one: bool) {
        let text = |field: &str| entry.get(field).and_then(Value::as_str);
        let flag = |field: &str| entry.get(field).and_then(Value::as_bool) == Some(true);
        let installed_name = key
            .rsplit_once(INSTALL_FOLDER)
            .map_or(key, |(_, name)| name);
        let version = text("version").unwrap_or_default();
        let (name, version) = match version.strip_prefix("npm:") {
            Some(alias) if is_version_one => match alias.rsplit_once('@') {
                Some((name, version)) if !name.is_empty() => (name, version),
                _ => (installed_name, alias),
            },
            _ => (text("name").unwrap_or(installed_name), version),
        };
        let resolved = text("resolved").or_else(|| {
            let reference = version;
            (is_version_one
                && !reference
                    .chars()
                    .next()
                    .is_some_and(|first| first.is_ascii_digit()))
            .then_some(reference)
        });
        let source = if flag("inBundle") || flag("bundled") {
            PackageSource::Bundled
        } else if flag("link") {
            PackageSource::Folder(
                resolved
                    .unwrap_or(key)
                    .trim_start_matches("file:")
                    .to_string(),
            )
        } else {
            source_of(Located {
                name,
                version,
                resolved,
                integrity: text("integrity"),
            })
        };
        self.packages.insert(
            key.to_string(),
            LockedPackage {
                name: name.to_string(),
                version: version.to_string(),
                source,
            },
        );
    }
}

struct Located<'a> {
    name: &'a str,
    version: &'a str,
    resolved: Option<&'a str>,
    integrity: Option<&'a str>,
}

fn source_of(located: Located) -> PackageSource {
    let Some(resolved) = located.resolved else {
        return PackageSource::Registry {
            url: default_url(located.name, located.version),
            integrity: located.integrity.map(str::to_string),
        };
    };
    if let Some(folder) = resolved.strip_prefix("file:") {
        return if folder.ends_with(".tgz") || folder.ends_with(".tar.gz") {
            PackageSource::Unsupported(resolved.to_string())
        } else {
            PackageSource::Folder(folder.to_string())
        };
    }
    let is_git = ["git+", "git:", "github:", "gitlab:", "bitbucket:"]
        .iter()
        .any(|prefix| resolved.starts_with(prefix));
    if is_git {
        return PackageSource::Git(resolved.to_string());
    }
    if resolved.starts_with("https://") || resolved.starts_with("http://") {
        return PackageSource::Registry {
            url: resolved.to_string(),
            integrity: located.integrity.map(str::to_string),
        };
    }
    PackageSource::Unsupported(resolved.to_string())
}

fn install_candidates(folder: &str, name: &str) -> Vec<String> {
    let parts: Vec<&str> = folder.split('/').filter(|part| !part.is_empty()).collect();
    let mut candidates = Vec::new();
    for length in (0..=parts.len()).rev() {
        let prefix = &parts[..length];
        if prefix.last() == Some(&"node_modules") {
            continue;
        }
        candidates.push(match prefix {
            [] => format!("node_modules/{name}"),
            _ => format!("{}/node_modules/{name}", prefix.join("/")),
        });
    }
    candidates
}
