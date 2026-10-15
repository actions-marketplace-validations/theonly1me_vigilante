use std::path::PathBuf;

use crate::Error;
use crate::registry::{Archive, Checksum, Download, FetchRequest, Fetched, PackageId, Registry};

const NAME: &str = "typescript";
const VERSION: &str = "6.0.3";
const URL: &str = "https://registry.npmjs.org/typescript/-/typescript-6.0.3.tgz";
const INTEGRITY: &str = "sha512-y2TvuxSZPDyQakkFRPZHKFm+KKVqIisdg9/CZwm9ftvKXLP8NRWj38/ODjNbr43SsoXqNuAisEf1GdCxqWcdBw==";
const COMPILER: &str = "package/lib/typescript.js";
const LIBRARY_PREFIX: &str = "package/lib/lib.";
const LIBRARY_SUFFIX: &str = ".d.ts";

pub fn typescript(registry: &Registry) -> Result<Option<PathBuf>, Error> {
    let download = Download {
        url: URL.to_string(),
        checksum: Checksum::from_integrity(INTEGRITY),
        archive: Archive::TarGzip,
    };
    let fetched = registry.fetch(FetchRequest {
        package: PackageId {
            ecosystem: "npm",
            name: NAME,
            version: VERSION,
        },
        download: &download,
        keep: &is_kept,
    })?;
    Ok(match fetched {
        Fetched::Folder(folder) => Some(folder.join(COMPILER)),
        _ => None,
    })
}

fn is_kept(path: &str) -> bool {
    path == COMPILER || (path.starts_with(LIBRARY_PREFIX) && path.ends_with(LIBRARY_SUFFIX))
}
