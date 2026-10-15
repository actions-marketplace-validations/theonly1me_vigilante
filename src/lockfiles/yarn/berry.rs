use std::collections::HashMap;

use super::{YarnLockfile, YarnPackage, split_descriptor, unquoted};
use crate::Error;
use crate::document::Node;
use crate::lockfiles::{LockedPackage, PackageSource, Release, default_url};

const METADATA: &str = "__metadata";

pub fn parse(text: &str) -> Result<YarnLockfile, Error> {
    let document =
        Node::parse(text).map_err(|error| Error::new(format!("invalid yarn lockfile: {error}")))?;
    let mut lockfile = YarnLockfile::default();
    for entry in document.entries() {
        if entry.key == METADATA {
            continue;
        }
        let descriptors: Vec<String> = entry
            .key
            .split(", ")
            .map(|descriptor| unquoted(descriptor).to_string())
            .collect();
        let field = |name: &str| entry.value.get(name).and_then(Node::text).map(unquoted);
        let Some(resolution) = field("resolution") else {
            continue;
        };
        let Some((name, reference)) = split_descriptor(resolution) else {
            continue;
        };
        let dependencies: HashMap<String, String> = entry
            .value
            .get("dependencies")
            .map(Node::entries)
            .unwrap_or_default()
            .iter()
            .filter_map(|dependency| {
                let range = dependency.value.text().map(unquoted)?;
                Some((
                    dependency.key.clone(),
                    format!("{}@{range}", dependency.key),
                ))
            })
            .collect();
        if let Some(folder) = reference.strip_prefix("workspace:") {
            lockfile
                .workspace_dependencies
                .insert(folder.to_string(), dependencies.clone());
            lockfile
                .workspace_folders
                .insert(name.to_string(), folder.to_string());
        }
        let version = field("version").unwrap_or_default().to_string();
        lockfile.add(YarnPackage {
            descriptors,
            key: resolution.to_string(),
            package: LockedPackage {
                name: name.to_string(),
                version: version.clone(),
                source: source_of(Resolution { name, reference }),
            },
            dependencies,
        });
    }
    Ok(lockfile)
}

struct Resolution<'a> {
    name: &'a str,
    reference: &'a str,
}

fn source_of(resolution: Resolution) -> PackageSource {
    let Resolution { name, reference } = resolution;
    if let Some(inner) = reference.strip_prefix("patch:") {
        let unpatched = inner.split('#').next().unwrap_or(inner).replace("%3A", ":");
        if let Some((patched_name, patched_reference)) = split_descriptor(&unpatched) {
            return source_of(Resolution {
                name: patched_name,
                reference: patched_reference,
            });
        }
    }
    if let Some(npm_version) = reference.strip_prefix("npm:") {
        return PackageSource::Registry {
            url: default_url(Release {
                name,
                version: npm_version,
            }),
            integrity: None,
        };
    }
    for prefix in ["workspace:", "portal:", "link:"] {
        if let Some(folder) = reference.strip_prefix(prefix) {
            return PackageSource::Folder(folder.to_string());
        }
    }
    if reference.contains("github.com") && reference.contains('#') {
        return PackageSource::Git(reference.to_string());
    }
    PackageSource::Unsupported(reference.to_string())
}
