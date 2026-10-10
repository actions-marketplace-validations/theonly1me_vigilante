use super::{INDEX, Served, serve};
use crate::registry::test_server::{Symlink, gem, tar_gzip, tar_with_symlink, zip};
use crate::registry::{Archive, Download};

#[test]
fn a_tar_path_outside_the_package_stops_the_fetch() {
    let setup = serve(Served {
        path: "/package",
        bytes: tar_gzip(&[("../evil.js", INDEX)]),
    });

    let error = setup.fetch(setup.download(None)).expect_err("outside");

    assert!(error.to_string().contains("outside the package"), "{error}");
    assert!(!setup.cache.path().join("npm/evil.js").exists());
}

#[test]
fn a_symbolic_link_is_not_followed() {
    let setup = serve(Served {
        path: "/package",
        bytes: tar_with_symlink(Symlink {
            link: "package/link.js",
            target: "/etc/passwd",
        }),
    });

    let folder = setup.fetch(setup.download(None)).expect("fetch");

    assert!(!folder.join("package/link.js").exists());
}

#[test]
fn a_zip_path_outside_the_package_stops_the_fetch() {
    let setup = serve(Served {
        path: "/package",
        bytes: zip(&[("../evil.py", INDEX)]),
    });

    let error = setup
        .fetch(Download {
            url: format!("{}/package", setup.server.base_url),
            checksum: None,
            archive: Archive::Zip,
        })
        .expect_err("outside");

    assert!(error.to_string().contains("outside the package"), "{error}");
}

#[test]
fn a_gem_is_read_through_its_data_archive_and_paths_stay_inside() {
    let setup = serve(Served {
        path: "/package",
        bytes: gem(&[("lib/tool.js", INDEX), ("../evil.js", INDEX)]),
    });

    let error = setup
        .fetch(Download {
            url: format!("{}/package", setup.server.base_url),
            checksum: None,
            archive: Archive::Gem,
        })
        .expect_err("outside");

    assert!(error.to_string().contains("outside the package"), "{error}");
}

#[test]
fn a_gem_keeps_the_files_of_its_data_archive() {
    let setup = serve(Served {
        path: "/package",
        bytes: gem(&[("lib/tool.js", INDEX)]),
    });

    let folder = setup
        .fetch(Download {
            url: format!("{}/package", setup.server.base_url),
            checksum: None,
            archive: Archive::Gem,
        })
        .expect("fetch");

    assert_eq!(
        std::fs::read(folder.join("lib/tool.js")).expect("file"),
        INDEX
    );
}
