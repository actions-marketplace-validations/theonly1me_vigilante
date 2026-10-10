use std::collections::BTreeMap;
use std::fs;

use tempfile::TempDir;

use super::{INDEX, Served, Setup, integrity, serve};
use crate::registry::test_server::{Serving, TestServer, tar_gzip};
use crate::registry::{Archive, Download, Registry};

#[test]
fn a_package_is_verified_and_only_wanted_files_are_kept() {
    let bytes = tar_gzip(&[
        ("package/index.js", INDEX),
        ("package/README.md", b"# tool\n"),
        ("package/lib/run.js", INDEX),
    ]);
    let setup = serve(Served {
        path: "/package",
        bytes: bytes.clone(),
    });

    let folder = setup
        .fetch(setup.download(Some(integrity(&bytes))))
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
    let setup = serve(Served {
        path: "/package",
        bytes: bytes.clone(),
    });
    let wanted = setup.download(Some(integrity(&bytes)));

    setup.fetch(wanted.clone()).expect("first fetch");
    setup.fetch(wanted).expect("second fetch");

    assert_eq!(setup.server.request_count(), 1);
}

#[test]
fn a_wrong_checksum_stops_and_caches_nothing() {
    let setup = serve(Served {
        path: "/package",
        bytes: tar_gzip(&[("package/index.js", INDEX)]),
    });
    let wrong = integrity(b"another package");

    let error = setup
        .fetch(setup.download(Some(wrong)))
        .expect_err("checksum");

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
fn a_missing_package_is_not_found() {
    let setup = serve(Served {
        path: "/elsewhere",
        bytes: Vec::new(),
    });

    let error = setup.fetch(setup.download(None)).expect_err("missing");

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

    setup.fetch(at("/first", &first)).expect("first fetch");
    let folder = setup.fetch(at("/second", &second)).expect("second fetch");

    assert_eq!(setup.server.request_count(), 2);
    assert_eq!(
        fs::read(folder.join("package/index.js")).expect("index"),
        b"module.exports = 2;\n"
    );
}

#[test]
fn a_server_error_is_tried_again() {
    let bytes = tar_gzip(&[("package/index.js", INDEX)]);
    let server = TestServer::serve_with(Serving {
        files: BTreeMap::from([("/package".to_string(), bytes.clone())]),
        failures: 1,
    });
    let cache = TempDir::new().expect("cache");
    let setup = Setup {
        registry: Registry::new(cache.path().to_path_buf()),
        server,
        cache,
    };

    let folder = setup
        .fetch(setup.download(Some(integrity(&bytes))))
        .expect("fetch");

    assert_eq!(setup.server.request_count(), 2);
    assert!(folder.join("package/index.js").is_file());
}
