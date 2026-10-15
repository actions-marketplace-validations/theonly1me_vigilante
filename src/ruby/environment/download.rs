use serde_json::Value;

use super::RubyEnvironment;
use crate::Error;
use crate::registry::{Archive, Checksum, Download, github_archive};
use crate::ruby::bundler::{GemSource, LockedGem};

const PLAIN_PLATFORM: &str = "ruby";

pub enum Planned {
    Download(Download),
    Skip(String),
}

impl RubyEnvironment<'_> {
    pub(super) fn download_of(&self, gem: &LockedGem) -> Result<Planned, Error> {
        let label = format!("`{} {}`", gem.name, gem.version);
        match &gem.source {
            GemSource::Path { .. } => Ok(Planned::Skip(format!(
                "{label} is a path gem outside the repository, so Vigilante did not read it."
            ))),
            GemSource::Git { remote, revision } => {
                Ok(match github_archive(&format!("git+{remote}#{revision}")) {
                    Some(archive) => Planned::Download(archive.download),
                    None => Planned::Skip(format!(
                        "{label} comes from `{remote}`, a git repository outside GitHub, so Vigilante did not read it."
                    )),
                })
            }
            GemSource::Registry { remote } => {
                let remote = format!("{}/", remote.trim_end_matches('/'));
                if !self.sources.allows(&remote) {
                    return Ok(Planned::Skip(format!(
                        "The registry `{remote}` is not public. Add it to `[registries] allow` in vigilante.toml if it needs no credentials."
                    )));
                }
                let file = match &gem.platform {
                    Some(platform) => format!("{}-{}-{platform}", gem.name, gem.version),
                    None => format!("{}-{}", gem.name, gem.version),
                };
                let checksum = match &gem.checksum {
                    Some(hex) => Checksum::sha256_hex(hex),
                    None => self.published_checksum(PublishedGem {
                        remote: &remote,
                        gem,
                    })?,
                };
                let Some(checksum) = checksum else {
                    return Ok(Planned::Skip(format!(
                        "{label} has no checksum, so Vigilante did not read it."
                    )));
                };
                Ok(Planned::Download(Download {
                    url: format!("{remote}downloads/{file}.gem"),
                    checksum: Some(checksum),
                    archive: Archive::Gem,
                }))
            }
        }
    }

    fn published_checksum(&self, published: PublishedGem) -> Result<Option<Checksum>, Error> {
        let PublishedGem { remote, gem } = published;
        let platform = gem.platform.as_deref().unwrap_or(PLAIN_PLATFORM);
        let address = format!(
            "{remote}api/v2/rubygems/{}/versions/{}.json?platform={platform}",
            gem.name, gem.version
        );
        Ok(self
            .registry
            .get_json(&address)?
            .and_then(|document| {
                document
                    .get("sha")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .and_then(|hex| Checksum::sha256_hex(&hex)))
    }
}

struct PublishedGem<'a> {
    remote: &'a str,
    gem: &'a LockedGem,
}
