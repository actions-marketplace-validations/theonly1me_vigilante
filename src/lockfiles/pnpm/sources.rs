use super::Format;
use crate::document::Node;
use crate::lockfiles::{PackageSource, Release, default_url};

impl Format {
    pub fn name_and_version(self, key: &str) -> Option<(String, String)> {
        name_and_version(Keyed { key, format: self })
    }
}

pub struct PackageEntry<'a> {
    pub key: &'a str,
    pub format: Format,
    pub entry: &'a Node,
}

struct Keyed<'a> {
    key: &'a str,
    format: Format,
}

fn name_and_version(keyed: Keyed) -> Option<(String, String)> {
    let Keyed { key, format } = keyed;
    let plain = key.trim_start_matches('/');
    match format {
        Format::Slash => {
            let (name, version) = plain.rsplit_once('/')?;
            Some((name.to_string(), version.split('_').next()?.to_string()))
        }
        Format::SlashAt | Format::At => {
            let without_peers = plain.split('(').next()?;
            let (name, version) = without_peers.rsplit_once('@')?;
            (!name.is_empty()).then(|| (name.to_string(), version.to_string()))
        }
    }
}

pub fn source_of(package: PackageEntry) -> PackageSource {
    let PackageEntry { key, format, entry } = package;
    let resolution = entry.get("resolution");
    let field = |name: &str| {
        resolution
            .and_then(|resolution| resolution.get(name))
            .and_then(Node::text)
    };
    let integrity = field("integrity").map(str::to_string);
    if let (Some(repository), Some(commit)) = (field("repo"), field("commit")) {
        return PackageSource::Git(format!("{repository}#{commit}"));
    }
    if let Some(folder) = field("directory") {
        return PackageSource::Folder(folder.to_string());
    }
    if let Some(tarball) = field("tarball") {
        return if tarball.starts_with("https://") || tarball.starts_with("http://") {
            PackageSource::Registry {
                url: tarball.to_string(),
                integrity,
            }
        } else {
            PackageSource::Unsupported(tarball.to_string())
        };
    }
    match format.name_and_version(key) {
        Some((name, version)) => PackageSource::Registry {
            url: default_url(Release {
                name: &name,
                version: &version,
            }),
            integrity,
        },
        None => PackageSource::Unsupported(key.to_string()),
    }
}
