mod cache;
mod checksum;
mod download;
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

use cache::{Cache, CacheFill, CacheKey};
pub use checksum::Checksum;
use download::{Downloaded, download};
use extract::{ExtractRequest, extract};
pub use git::github_archive;
pub use sources::Sources;

use sha2::{Digest, Sha256};

use crate::Error;

const TIMEOUT: Duration = Duration::from_secs(60);
const IDENTITY_LENGTH: usize = 16;

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

pub enum Fetched {
    Folder(PathBuf),
    NotFound,
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

    pub fn fetch(&self, request: FetchRequest) -> Result<Fetched, Error> {
        let identity = identity_of(request.download);
        let key = CacheKey {
            package: &request.package,
            identity: &identity,
        };
        if let Some(folder) = self.cache.complete_folder(&key) {
            return Ok(Fetched::Folder(folder));
        }
        let url = &request.download.url;
        let Downloaded::Bytes(bytes) = download(&self.agent, url)? else {
            return Ok(Fetched::NotFound);
        };
        if let Some(checksum) = &request.download.checksum
            && !checksum.matches(&bytes)
        {
            return Err(Error::new(format!(
                "{url} does not match the checksum in the lockfile"
            )));
        }
        self.cache
            .fill(CacheFill {
                key: &key,
                fill: &|target| {
                    extract(ExtractRequest {
                        bytes: &bytes,
                        archive: request.download.archive,
                        target,
                        keep: request.keep,
                    })
                },
            })
            .map(Fetched::Folder)
    }
}

fn identity_of(download: &Download) -> String {
    let digest = Sha256::digest(format!("{}\n{:?}", download.url, download.checksum));
    digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
        .chars()
        .take(IDENTITY_LENGTH)
        .collect()
}
