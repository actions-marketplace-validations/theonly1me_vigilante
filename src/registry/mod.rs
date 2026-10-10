mod cache;
mod checksum;
mod extract;
mod git;
mod sources;
#[cfg(test)]
mod test_server;
#[cfg(test)]
mod tests;

use std::env;
use std::path::PathBuf;
use std::time::Duration;

use cache::{Cache, CacheFill};
pub use checksum::Checksum;
use extract::{ExtractRequest, extract};
pub use git::github_archive;
pub use sources::Sources;

use crate::Error;

const MAXIMUM_DOWNLOAD_BYTES: u64 = 256 * 1024 * 1024;
const TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Archive {
    TarGzip,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Download {
    pub url: String,
    pub checksum: Option<Checksum>,
    pub archive: Archive,
}

pub struct PackageId<'a> {
    pub ecosystem: &'a str,
    pub name: &'a str,
    pub version: &'a str,
}

pub struct FetchRequest<'a> {
    pub package: PackageId<'a>,
    pub download: &'a Download,
    pub keep: &'a dyn Fn(&str) -> bool,
}

pub struct Registry {
    cache: Cache,
    agent: ureq::Agent,
}

impl Registry {
    pub fn new(cache_root: PathBuf) -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(TIMEOUT))
            .user_agent(concat!("vigilante/", env!("CARGO_PKG_VERSION")))
            .build()
            .new_agent();
        Self {
            cache: Cache::new(cache_root),
            agent,
        }
    }

    pub fn default_cache_root() -> PathBuf {
        let base = env::var_os("XDG_CACHE_HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                let home = PathBuf::from(env::var_os("HOME")?);
                Some(if cfg!(target_os = "macos") {
                    home.join("Library").join("Caches")
                } else {
                    home.join(".cache")
                })
            })
            .unwrap_or_else(env::temp_dir);
        base.join("vigilante")
    }

    pub fn fetch(&self, request: FetchRequest) -> Result<PathBuf, Error> {
        if let Some(folder) = self.cache.complete_folder(&request.package) {
            return Ok(folder);
        }
        let url = &request.download.url;
        let bytes = self
            .agent
            .get(url)
            .call()
            .and_then(|mut response| {
                response
                    .body_mut()
                    .with_config()
                    .limit(MAXIMUM_DOWNLOAD_BYTES)
                    .read_to_vec()
            })
            .map_err(|error| Error::new(format!("cannot download {url}: {error}")))?;
        if let Some(checksum) = &request.download.checksum
            && !checksum.matches(&bytes)
        {
            return Err(Error::new(format!(
                "{url} does not match the checksum in the lockfile"
            )));
        }
        self.cache.fill(CacheFill {
            package: &request.package,
            fill: &|target| {
                extract(ExtractRequest {
                    bytes: &bytes,
                    archive: request.download.archive,
                    target,
                    keep: request.keep,
                })
            },
        })
    }
}
