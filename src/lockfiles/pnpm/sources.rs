use super::Format;
use crate::document::Node;
use crate::lockfiles::{PackageSource, default_url};

pub fn name_and_version(key: &str, format: Format) -> Option<(String, String)> {
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

pub fn source_of(key: &str, format: Format, entry: &Node) -> PackageSource {
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
    match name_and_version(key, format) {
        Some((name, version)) => PackageSource::Registry {
            url: default_url(&name, &version),
            integrity,
        },
        None => PackageSource::Unsupported(key.to_string()),
    }
}
