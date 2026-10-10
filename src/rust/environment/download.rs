use serde_json::Value;

use super::RustEnvironment;
use crate::Error;
use crate::registry::{Archive, Checksum, Download, github_archive};
use crate::rust::cargo::LockedPackage;

const CRATES_IO_SOURCES: &[&str] = &[
    "registry+https://github.com/rust-lang/crates.io-index",
    "sparse+https://index.crates.io/",
];
const CRATES_IO_DOWNLOADS: &str = "https://static.crates.io/crates";
const TEMPLATE_MARKERS: &[&str] = &[
    "{crate}",
    "{version}",
    "{prefix}",
    "{lowerprefix}",
    "{sha256-checksum}",
];

pub enum Planned {
    Download(Download),
    Skip(String),
}

impl RustEnvironment<'_> {
    pub(super) fn download_of(&self, package: &LockedPackage) -> Result<Planned, Error> {
        let source = package.source.as_deref().unwrap_or_default();
        let label = format!("`{} {}`", package.name, package.version);
        let not_public = |address: &str| {
            Planned::Skip(format!(
                "The registry `{address}` is not public. Add it to `[registries] allow` in vigilante.toml if it needs no credentials."
            ))
        };
        if let Some(repository) = source.strip_prefix("git+") {
            return Ok(match github_archive(source) {
                Some(archive) => Planned::Download(archive.download),
                None => Planned::Skip(format!(
                    "{label} comes from `{repository}`, a git repository outside GitHub, so Vigilante did not read it."
                )),
            });
        }
        let Some(checksum) = package.checksum.as_deref().and_then(Checksum::sha256_hex) else {
            return Ok(Planned::Skip(format!(
                "{label} has no checksum in Cargo.lock, so Vigilante did not read it."
            )));
        };
        let template = if CRATES_IO_SOURCES.contains(&source) {
            format!("{CRATES_IO_DOWNLOADS}/{{crate}}/{{crate}}-{{version}}.crate")
        } else if let Some(index) = source.strip_prefix("sparse+") {
            if !self.sources.allows(index) {
                return Ok(not_public(index));
            }
            let config = self
                .registry
                .get_json(&format!("{}/config.json", index.trim_end_matches('/')))?;
            match config
                .as_ref()
                .and_then(|config| config.get("dl"))
                .and_then(Value::as_str)
            {
                Some(template) => template.to_string(),
                None => return Ok(not_public(index)),
            }
        } else {
            return Ok(not_public(source));
        };
        let url = download_url(DownloadTemplate {
            template: &template,
            package,
        });
        if !self.sources.allows(&url) {
            return Ok(not_public(&url));
        }
        Ok(Planned::Download(Download {
            url,
            checksum: Some(checksum),
            archive: Archive::TarGzip,
        }))
    }
}

struct DownloadTemplate<'a> {
    template: &'a str,
    package: &'a LockedPackage,
}

fn download_url(filled: DownloadTemplate) -> String {
    let DownloadTemplate { template, package } = filled;
    if !TEMPLATE_MARKERS
        .iter()
        .any(|marker| template.contains(marker))
    {
        return format!(
            "{}/{}/{}/download",
            template.trim_end_matches('/'),
            package.name,
            package.version
        );
    }
    let name = &package.name;
    let prefix = match name.len() {
        1 => "1".to_string(),
        2 => "2".to_string(),
        3 => format!("3/{}", &name[..1]),
        _ => format!("{}/{}", &name[..2], &name[2..4]),
    };
    template
        .replace("{crate}", name)
        .replace("{version}", &package.version)
        .replace("{prefix}", &prefix)
        .replace("{lowerprefix}", &prefix.to_lowercase())
        .replace(
            "{sha256-checksum}",
            package.checksum.as_deref().unwrap_or_default(),
        )
}
