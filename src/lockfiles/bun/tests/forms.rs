use super::{Query, ROOT, bun, found};
use crate::lockfiles::{JavaScriptLockfile, LockfileText, PackageSource, Place};

#[test]
fn a_scoped_parent_is_one_step_of_the_nested_path() {
    let lockfile = bun(r#"{
      "packages": {
        "@scope/a": ["@scope/a@1.0.0", "", {}, ""],
        "@scope/c": ["@scope/c@1.0.0", "", {}, ""],
        "c": ["c@2.0.0", "", {}, ""],
        "@acme/web/runner": ["runner@3.0.0", "", {}, ""],
        "@acme/web": ["@acme/web@workspace:apps/web"],
      },
    }"#);

    assert_eq!(
        found(Query {
            lockfile: &lockfile,
            place: Place::Package { key: "@scope/a" },
            name: "c",
        })
        .1,
        "c@2.0.0"
    );
    assert_eq!(
        found(Query {
            lockfile: &lockfile,
            place: Place::Repository { folder: "apps/web" },
            name: "runner",
        })
        .0,
        "@acme/web/runner"
    );
}

#[test]
fn a_private_registry_keeps_its_tarball_address() {
    let lockfile = bun(r#"{"packages": {
      "private": ["private@1.0.0", "https://npm.example.com/private/-/private-1.0.0.tgz", {}, "sha512-AAAA"],
    }}"#);

    assert_eq!(
        found(Query {
            lockfile: &lockfile,
            place: ROOT,
            name: "private",
        })
        .2,
        PackageSource::Registry {
            url: "https://npm.example.com/private/-/private-1.0.0.tgz".to_string(),
            integrity: Some("sha512-AAAA".to_string()),
        }
    );
}

#[test]
fn global_links_and_local_tarballs_are_not_read() {
    let lockfile = bun(r#"{"packages": {
      "linked": ["linked@link:linked", {}],
      "vendored": ["vendored@./vendor/vendored-1.0.0.tgz", {}],
      "packed": ["packed@file:vendor/packed-1.0.0.tgz", {}],
    }}"#);
    let source_of = |name| {
        found(Query {
            lockfile: &lockfile,
            place: ROOT,
            name,
        })
        .2
    };

    assert_eq!(
        [
            source_of("linked"),
            source_of("vendored"),
            source_of("packed")
        ],
        [
            PackageSource::Unsupported("link:linked".to_string()),
            PackageSource::Unsupported("./vendor/vendored-1.0.0.tgz".to_string()),
            PackageSource::Unsupported("file:vendor/packed-1.0.0.tgz".to_string()),
        ]
    );
}

#[test]
fn comments_and_trailing_commas_outside_strings_are_ignored() {
    let lockfile = bun(r#"// written by hand
    {
      /* the packages */
      "packages": {
        "odd": ["odd@1.0.0", "https://example.com/a//b,]/*c*/odd.tgz", {}, "sha512-\"q\",}",], // odd
      },
    }"#);

    assert_eq!(
        found(Query {
            lockfile: &lockfile,
            place: ROOT,
            name: "odd",
        })
        .2,
        PackageSource::Registry {
            url: "https://example.com/a//b,]/*c*/odd.tgz".to_string(),
            integrity: Some("sha512-\"q\",}".to_string()),
        }
    );
}

#[test]
fn a_package_without_a_resolution_stops_the_run() {
    let error = JavaScriptLockfile::parse(LockfileText {
        path: "bun.lock",
        text: r#"{"packages": {"ghost": {"version": "1.0.0"}}}"#,
    })
    .err()
    .expect("an error");

    assert_eq!(
        error.to_string(),
        "invalid bun lockfile: the package `ghost` has no resolution"
    );
}
