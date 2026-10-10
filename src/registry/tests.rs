use std::collections::BTreeMap;
use std::fs;

use base64::Engine;
use sha2::{Digest, Sha512};
use tempfile::TempDir;

use super::test_server::{TestServer, tar_gzip, tar_with_symlink};
use super::{Archive, Checksum, Download, FetchRequest, Fetched, PackageId, Registry};
use crate::Error;

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

fn serve(path: &str, bytes: Vec<u8>) -> Setup {
    let server = TestServer::serve(BTreeMap::from([(path.to_string(), bytes)]));
    let cache = TempDir::new().expect("cache");
    let registry = Registry::new(cache.path().to_path_buf());
    Setup {
        server,
        cache,
        registry,
    }
}

fn fetch(setup: &Setup, download: Download) -> Result<std::path::PathBuf, Error> {
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
    }
}

fn download(setup: &Setup, archive: Archive, checksum: Option<Checksum>) -> Download {
    Download {
        url: format!("{}/package", setup.server.base_url),
        checksum,
        archive,
    }
}

#[test]
fn a_package_is_verified_and_only_wanted_files_are_kept() {
    let bytes = tar_gzip(&[
        ("package/index.js", INDEX),
        ("package/README.md", b"# tool\n"),
        ("package/lib/run.js", INDEX),
    ]);
    let setup = serve("/package", bytes.clone());

    let folder = fetch(
        &setup,
        download(&setup, Archive::TarGzip, Some(integrity(&bytes))),
    )
    .expect("fetch");

    assert_eq!(
        fs::read(folder.join("package/index.js")).expect("index"),
        INDEX
    );
    assert!(folder.join("package/lib/run.js").is_file());
    assert!(!folder.join("package/README.md").exists());
}

#[test]
fn a_second_fetch_reads_the_cache() {
    let bytes = tar_gzip(&[("package/index.js", INDEX)]);
    let setup = serve("/package", bytes.clone());
    let wanted = download(&setup, Archive::TarGzip, Some(integrity(&bytes)));

    fetch(&setup, wanted.clone()).expect("first fetch");
    fetch(&setup, wanted).expect("second fetch");

    assert_eq!(setup.server.request_count(), 1);
}

#[test]
fn a_wrong_checksum_stops_and_caches_nothing() {
    let setup = serve("/package", tar_gzip(&[("package/index.js", INDEX)]));
    let wrong = integrity(b"another package");

    let error =
        fetch(&setup, download(&setup, Archive::TarGzip, Some(wrong))).expect_err("checksum");

    assert!(
        error.to_string().contains("does not match the checksum"),
        "{error}"
    );
    assert_eq!(
        fs::read_dir(setup.cache.path())
            .map(|entries| entries.count())
            .unwrap_or(0),
        0
    );
}

#[test]
fn a_tar_path_outside_the_package_stops_the_fetch() {
    let setup = serve("/package", tar_gzip(&[("../evil.js", INDEX)]));

    let error = fetch(&setup, download(&setup, Archive::TarGzip, None)).expect_err("outside");

    assert!(error.to_string().contains("outside the package"), "{error}");
    assert!(!setup.cache.path().join("npm/evil.js").exists());
}

#[test]
fn a_symbolic_link_is_not_followed() {
    let setup = serve(
        "/package",
        tar_with_symlink("package/link.js", "/etc/passwd"),
    );

    let folder = fetch(&setup, download(&setup, Archive::TarGzip, None)).expect("fetch");

    assert!(!folder.join("package/link.js").exists());
}

#[test]
fn a_missing_package_is_not_found() {
    let setup = serve("/elsewhere", Vec::new());

    let error = fetch(&setup, download(&setup, Archive::TarGzip, None)).expect_err("missing");

    assert_eq!(error.to_string(), "not found");
}

#[test]
fn the_same_version_with_another_checksum_is_downloaded_again() {
    let first = tar_gzip(&[("package/index.js", INDEX)]);
    let second = tar_gzip(&[("package/index.js", b"module.exports = 2;\n")]);
    let server = TestServer::serve(BTreeMap::from([
        ("/first".to_string(), first.clone()),
        ("/second".to_string(), second.clone()),
    ]));
    let cache = TempDir::new().expect("cache");
    let setup = Setup {
        registry: Registry::new(cache.path().to_path_buf()),
        server,
        cache,
    };
    let at = |path: &str, bytes: &[u8]| Download {
        url: format!("{}{path}", setup.server.base_url),
        checksum: Some(integrity(bytes)),
        archive: Archive::TarGzip,
    };

    fetch(&setup, at("/first", &first)).expect("first fetch");
    let folder = fetch(&setup, at("/second", &second)).expect("second fetch");

    assert_eq!(setup.server.request_count(), 2);
    assert_eq!(
        fs::read(folder.join("package/index.js")).expect("index"),
        b"module.exports = 2;\n"
    );
}

#[test]
fn a_server_error_is_tried_again() {
    let bytes = tar_gzip(&[("package/index.js", INDEX)]);
    let server = TestServer::serve_after_failures(
        BTreeMap::from([("/package".to_string(), bytes.clone())]),
        1,
    );
    let cache = TempDir::new().expect("cache");
    let setup = Setup {
        registry: Registry::new(cache.path().to_path_buf()),
        server,
        cache,
    };

    let folder = fetch(
        &setup,
        download(&setup, Archive::TarGzip, Some(integrity(&bytes))),
    )
    .expect("fetch");

    assert_eq!(setup.server.request_count(), 2);
    assert!(folder.join("package/index.js").is_file());
}
