use std::collections::BTreeMap;

use base64::Engine;
use sha2::{Digest, Sha512};
use tempfile::TempDir;

use super::test_server::TestServer;
use super::{Archive, Checksum, Download, FetchRequest, Fetched, PackageId, Registry};
use crate::Error;

mod archives;
mod downloads;
mod sources;

const INDEX: &[u8] = b"module.exports = require('child_process')\n";

fn integrity(bytes: &[u8]) -> Checksum {
    let digest = base64::engine::general_purpose::STANDARD.encode(Sha512::digest(bytes));
    Checksum::from_integrity(&format!("sha512-{digest}")).expect("integrity")
}

struct Setup {
    server: TestServer,
    cache: TempDir,
    registry: Registry,
}

struct Served<'a> {
    path: &'a str,
    bytes: Vec<u8>,
}

fn serve(served: Served) -> Setup {
    let server = TestServer::serve(BTreeMap::from([(served.path.to_string(), served.bytes)]));
    let cache = TempDir::new().expect("cache");
    let registry = Registry::new(cache.path().to_path_buf());
    Setup {
        server,
        cache,
        registry,
    }
}

impl Setup {
    fn fetch(&self, download: Download) -> Result<std::path::PathBuf, Error> {
        fetch(Fetch {
            setup: self,
            download,
        })
    }

    fn download(&self, checksum: Option<Checksum>) -> Download {
        Download {
            url: format!("{}/package", self.server.base_url),
            checksum,
            archive: Archive::TarGzip,
        }
    }
}

struct Fetch<'a> {
    setup: &'a Setup,
    download: Download,
}

fn fetch(request: Fetch) -> Result<std::path::PathBuf, Error> {
    let Fetch { setup, download } = request;
    let fetched = setup.registry.fetch(FetchRequest {
        package: PackageId {
            ecosystem: "npm",
            name: "@scope/tool",
            version: "1.0.0",
        },
        download: &download,
        keep: &|path| path.ends_with(".js"),
    })?;
    match fetched {
        Fetched::Folder(folder) => Ok(folder),
        Fetched::NotFound => Err(Error::new("not found")),
        Fetched::TooLarge => Err(Error::new("too large")),
    }
}
