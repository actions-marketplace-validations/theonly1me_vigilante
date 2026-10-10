use super::{INDEX, Served, serve};
use crate::registry::test_server::{Symlink, tar_gzip, tar_with_symlink};

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
