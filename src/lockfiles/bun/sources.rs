use serde_json::Value;

use crate::Error;
use crate::lockfiles::{LockedPackage, PackageSource, Release, default_url};

const ROOT: &str = "root:";
const WORKSPACE: &str = "workspace:";
const FOLDER: &str = "file:";
const GIT_PREFIXES: &[&str] = &["github:", "git+", "git://", "ssh://", "git@"];
const TARBALL_ENDINGS: &[&str] = &[".tgz", ".tar.gz"];

pub struct Row<'a> {
    pub key: &'a str,
    pub entry: &'a Value,
}

pub struct ReadRow {
    pub package: LockedPackage,
    pub local_folder: Option<LocalFolder>,
}

pub struct LocalFolder {
    pub folder: String,
    pub prefix: String,
}

struct Resolution<'a> {
    name: &'a str,
    reference: &'a str,
    items: &'a [Value],
}

pub fn read_row(row: Row) -> Result<ReadRow, Error> {
    let Row { key, entry } = row;
    let items = entry.as_array().map(Vec::as_slice).unwrap_or_default();
    let Some((name, reference)) = items
        .first()
        .and_then(Value::as_str)
        .and_then(split_resolution)
    else {
        return Err(Error::new(format!(
            "invalid bun lockfile: the package `{key}` has no resolution"
        )));
    };
    let source = source_of(Resolution {
        name,
        reference,
        items,
    });
    let prefix = if reference.starts_with(WORKSPACE) {
        Some(name)
    } else if reference.starts_with(FOLDER) {
        Some(key)
    } else {
        None
    };
    let local_folder = match (&source, prefix) {
        (PackageSource::Folder(folder), Some(prefix)) => Some(LocalFolder {
            folder: folder.clone(),
            prefix: prefix.to_string(),
        }),
        _ => None,
    };
    Ok(ReadRow {
        package: LockedPackage {
            name: name.to_string(),
            version: reference.to_string(),
            source,
        },
        local_folder,
    })
}

fn source_of(resolution: Resolution) -> PackageSource {
    let Resolution {
        name,
        reference,
        items,
    } = resolution;
    let text = |index: usize| {
        items
            .get(index)
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
    };
    if reference.starts_with(ROOT) {
        return PackageSource::Folder(String::new());
    }
    let is_tarball = TARBALL_ENDINGS
        .iter()
        .any(|ending| reference.ends_with(ending));
    for prefix in [WORKSPACE, FOLDER] {
        if let Some(folder) = reference.strip_prefix(prefix)
            && !is_tarball
        {
            return PackageSource::Folder(folder.to_string());
        }
    }
    if GIT_PREFIXES
        .iter()
        .any(|prefix| reference.starts_with(prefix))
    {
        return PackageSource::Git(reference.to_string());
    }
    if reference.starts_with("https://") || reference.starts_with("http://") {
        return PackageSource::Registry {
            url: reference.to_string(),
            integrity: text(2).map(str::to_string),
        };
    }
    if reference.starts_with(|first: char| first.is_ascii_digit()) {
        return PackageSource::Registry {
            url: text(1).map_or_else(
                || {
                    default_url(Release {
                        name,
                        version: reference,
                    })
                },
                str::to_string,
            ),
            integrity: text(3).map(str::to_string),
        };
    }
    PackageSource::Unsupported(reference.to_string())
}

fn split_resolution(resolution: &str) -> Option<(&str, &str)> {
    if let Some(reference) = resolution
        .strip_prefix('@')
        .filter(|rest| rest.starts_with(ROOT))
    {
        return Some(("", reference));
    }
    let search_from = usize::from(resolution.starts_with('@'));
    let at = resolution[search_from..].find('@')? + search_from;
    Some((&resolution[..at], &resolution[at + 1..]))
}
