use std::collections::BTreeMap;
use std::fs;

use tempfile::TempDir;

use super::layout::layout;
use super::plan::{Plan, PlanContext, plan};
use crate::lockfiles::python::{PythonPackage, PythonSource};
use crate::registry::test_server::TestServer;
use crate::registry::{Archive, Checksum, Registry, Sources};

const WHEEL_HASH: &str = "aa00";
const SDIST_HASH: &str = "bb11";
const LARGE_HASH: &str = "cc22";
const SMALL_HASH: &str = "dd33";

struct Indexed {
    server: TestServer,
    _cache: TempDir,
    registry: Registry,
    sources: Sources,
}

fn indexed() -> Indexed {
    let listing = |base: &str| {
        format!(
            r#"{{"urls": [{{"url": "{base}/runner-1.0.0.tar.gz", "size": 900, "digests": {{"sha256": "{SDIST_HASH}"}}}}, {{"url": "{base}/runner-1.0.0-cp312-manylinux_x86_64.whl", "size": 800000000, "digests": {{"sha256": "{LARGE_HASH}"}}}}, {{"url": "{base}/runner-1.0.0-cp312-macosx_11_0_arm64.whl", "size": 2000, "digests": {{"sha256": "{SMALL_HASH}"}}}}, {{"url": "{base}/runner-1.0.0-py3-none-any.whl", "digests": {{"sha256": "{WHEEL_HASH}"}}}}]}}"#
        )
    };
    let server = TestServer::serve_built(Box::new(move |base| {
        BTreeMap::from([(
            "/pypi/runner/1.0.0/json".to_string(),
            listing(base).into_bytes(),
        )])
    }));
    let cache = TempDir::new().expect("cache");
    let sources = Sources::new(std::slice::from_ref(&server.base_url)).expect("sources");
    Indexed {
        registry: Registry::new(cache.path().to_path_buf()),
        _cache: cache,
        sources,
        server,
    }
}

fn plan_with(hashes: &[&str]) -> Plan {
    let setup = indexed();
    plan(PlanContext {
        registry: &setup.registry,
        sources: &setup.sources,
        index: &format!("{}/pypi", setup.server.base_url),
        package: &PythonPackage {
            name: "runner".to_string(),
            version: "1.0.0".to_string(),
            source: PythonSource::Index {
                hashes: hashes.iter().map(|hash| hash.to_string()).collect(),
            },
        },
    })
    .unwrap_or_else(|error| Plan::Skip(error.to_string()))
}

#[test]
fn an_index_package_uses_the_wheel_whose_hash_the_lockfile_lists() {
    let Plan::Download(planned) = plan_with(&[SDIST_HASH, WHEEL_HASH]) else {
        panic!("no download");
    };

    assert!(
        planned
            .download
            .url
            .ends_with("/runner-1.0.0-py3-none-any.whl")
    );
    assert_eq!(planned.download.checksum, Checksum::sha256_hex(WHEEL_HASH));
    assert_eq!(planned.download.archive, Archive::Zip);
}

#[test]
fn an_index_package_without_a_pure_wheel_uses_its_smallest_wheel() {
    let Plan::Download(planned) = plan_with(&[SDIST_HASH, LARGE_HASH, SMALL_HASH]) else {
        panic!("no download");
    };

    assert!(
        planned
            .download
            .url
            .ends_with("/runner-1.0.0-cp312-macosx_11_0_arm64.whl")
    );
    assert_eq!(planned.download.archive, Archive::Zip);
}

#[test]
fn an_index_package_with_no_wheel_uses_its_source_archive() {
    let Plan::Download(planned) = plan_with(&[SDIST_HASH]) else {
        panic!("no download");
    };

    assert!(planned.download.url.ends_with("/runner-1.0.0.tar.gz"));
    assert_eq!(planned.download.archive, Archive::TarGzip);
}

#[test]
fn an_index_package_with_no_matching_hash_stops() {
    let Plan::Skip(reason) = plan_with(&["ffff"]) else {
        panic!("planned a download");
    };

    assert!(
        reason.contains("matches the hashes in the lockfile"),
        "{reason}"
    );
}

#[test]
fn a_source_archive_with_a_src_folder_finds_its_modules() {
    let folder = TempDir::new().expect("folder");
    for (path, text) in [
        ("runner-1.0.0/setup.py", ""),
        ("runner-1.0.0/src/runner/__init__.py", ""),
        ("runner-1.0.0/src/runner/core.py", ""),
        ("runner-1.0.0/src/helpers.py", ""),
    ] {
        let target = folder.path().join(path);
        fs::create_dir_all(target.parent().expect("parent")).expect("folders");
        fs::write(target, text).expect("file");
    }

    let found = layout(folder.path()).expect("layout");

    assert_eq!(found.module_roots, vec!["runner-1.0.0/src".to_string()]);
    assert_eq!(
        found.modules.into_iter().collect::<Vec<_>>(),
        vec!["helpers".to_string(), "runner".to_string()]
    );
}
