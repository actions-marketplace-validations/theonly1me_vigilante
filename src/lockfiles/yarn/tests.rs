use std::collections::HashMap;

use crate::lockfiles::{Found, JavaScriptLockfile, Lookup, PackageSource, Place};

const CLASSIC: &str = r#"# yarn lockfile v1


runner@^1.0.0:
  version "1.0.0"
  resolved "https://registry.yarnpkg.com/runner/-/runner-1.0.0.tgz#0123456789abcdef0123456789abcdef01234567"
  dependencies:
    spawner "^2.0.0"

spawner@^1.0.0:
  version "1.0.0"
  resolved "https://registry.yarnpkg.com/spawner/-/spawner-1.0.0.tgz"
  integrity sha512-AAAA

spawner@^2.0.0:
  version "2.0.0"
  resolved "https://registry.yarnpkg.com/spawner/-/spawner-2.0.0.tgz"
  integrity sha512-BBBB

"string-width-cjs@npm:string-width@^4.2.0":
  version "4.2.3"
  resolved "https://registry.yarnpkg.com/string-width/-/string-width-4.2.3.tgz"
  integrity sha512-CCCC
"#;

const BERRY: &str = r#"__metadata:
  version: 8
  cacheKey: 10c0

"app@workspace:.":
  version: 0.0.0-use.local
  resolution: "app@workspace:."
  dependencies:
    runner: "npm:^1.0.0"
    typescript: "patch:typescript@npm%3A^5.0.0#optional!builtin<compat/typescript>"
  languageName: unknown
  linkType: soft

"web@workspace:apps/web":
  version: 0.0.0-use.local
  resolution: "web@workspace:apps/web"
  languageName: unknown
  linkType: soft

"runner@npm:^1.0.0":
  version: 1.0.0
  resolution: "runner@npm:1.0.0"
  dependencies:
    spawner: "npm:^2.0.0"
  checksum: 10c0/abc
  languageName: node
  linkType: hard

"runner@npm:^2.0.0":
  version: 2.0.0
  resolution: "runner@npm:2.0.0"
  languageName: node
  linkType: hard

"spawner@npm:^2.0.0":
  version: 2.0.0
  resolution: "spawner@npm:2.0.0"
  languageName: node
  linkType: hard

"typescript@patch:typescript@npm%3A^5.0.0#optional!builtin<compat/typescript>":
  version: 5.0.4
  resolution: "typescript@patch:typescript@npm%3A5.0.4#optional!builtin<compat/typescript>::version=5.0.4&hash=b5f058"
  languageName: node
  linkType: hard
"#;

fn yarn(text: &str) -> JavaScriptLockfile {
    JavaScriptLockfile::parse("yarn.lock", text).expect("lockfile")
}

fn found(lockfile: &JavaScriptLockfile, lookup: Lookup) -> (String, String, PackageSource) {
    let Found { key, package } = lockfile.find(lookup).expect("found");
    (
        key,
        format!("{}@{}", package.name, package.version),
        package.source,
    )
}

fn from_repository<'a>(name: &'a str, declared: Option<&'a str>) -> Lookup<'a> {
    Lookup {
        place: Place::Repository { folder: "src" },
        name,
        declared,
    }
}

fn registry(url: &str, integrity: Option<&str>) -> PackageSource {
    PackageSource::Registry {
        url: url.to_string(),
        integrity: integrity.map(str::to_string),
    }
}

#[test]
fn version_1_reads_declared_ranges_dependencies_aliases_and_sha1_hashes() {
    let lockfile = yarn(CLASSIC);

    let runner = found(&lockfile, from_repository("runner", Some("^1.0.0")));
    let spawner = found(
        &lockfile,
        Lookup {
            place: Place::Package { key: &runner.0 },
            name: "spawner",
            declared: None,
        },
    );
    let alias = found(
        &lockfile,
        from_repository("string-width-cjs", Some("npm:string-width@^4.2.0")),
    );

    assert_eq!(
        runner.2,
        registry(
            "https://registry.yarnpkg.com/runner/-/runner-1.0.0.tgz",
            Some("sha1-ASNFZ4mrze8BI0VniavN7wEjRWc=")
        )
    );
    assert_eq!(spawner.1, "spawner@2.0.0");
    assert_eq!(alias.1, "string-width@4.2.3");
    assert_eq!(lockfile.find(from_repository("spawner", None)), None);
}

#[test]
fn version_1_finds_workspace_packages_by_name() {
    let mut lockfile = yarn(CLASSIC);
    lockfile.add_workspaces(HashMap::from([(
        "@acme/tools".to_string(),
        "packages/tools".to_string(),
    )]));

    let tools = found(&lockfile, from_repository("@acme/tools", Some("*")));

    assert_eq!(tools.2, PackageSource::Folder("packages/tools".to_string()));
}

#[test]
fn berry_reads_workspaces_dependencies_and_patched_packages() {
    let lockfile = yarn(BERRY);

    let runner = found(&lockfile, from_repository("runner", None));
    let spawner = found(
        &lockfile,
        Lookup {
            place: Place::Package { key: &runner.0 },
            name: "spawner",
            declared: None,
        },
    );
    let typescript = found(&lockfile, from_repository("typescript", None));
    let web = found(&lockfile, from_repository("web", None));

    assert_eq!(runner.1, "runner@1.0.0");
    assert_eq!(
        spawner.2,
        registry(
            "https://registry.npmjs.org/spawner/-/spawner-2.0.0.tgz",
            None
        )
    );
    assert_eq!(
        typescript.2,
        registry(
            "https://registry.npmjs.org/typescript/-/typescript-5.0.4.tgz",
            None
        )
    );
    assert_eq!(web.2, PackageSource::Folder("apps/web".to_string()));
}
