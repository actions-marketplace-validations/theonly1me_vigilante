use std::path::PathBuf;

use crate::Error;
use crate::registry::{Archive, Checksum, Download, FetchRequest, Fetched, PackageId, Registry};

const NAME: &str = "@sourcegraph/scip-python";
const VERSION: &str = "0.6.6";
const URL: &str = "https://registry.npmjs.org/@sourcegraph/scip-python/-/scip-python-0.6.6.tgz";
const INTEGRITY: &str = "sha512-qoKL1Rggg0o5newAFbCFAKlS0AjWxG5MA+mC28BtgxOv0DhO4zdL8u7151FxEppDpXMVvm7+yXSjXotoVH9cMQ==";
const ENTRY: &str = "package/index.js";
const BUNDLES: &[&str] = &[
    "package/dist/scip-python.js",
    "package/dist/pyright-internal.js",
    "package/dist/vendor.js",
];
const TYPESHED: &str = "package/dist/typeshed-fallback/";

pub fn scip_python(registry: &Registry) -> Result<Option<PathBuf>, Error> {
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
        Fetched::Folder(folder) => Some(folder.join(ENTRY)),
        Fetched::NotFound | Fetched::TooLarge => None,
    })
}

fn is_kept(path: &str) -> bool {
    path == ENTRY || BUNDLES.contains(&path) || path.starts_with(TYPESHED)
}
