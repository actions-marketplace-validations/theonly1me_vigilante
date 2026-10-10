use super::{Found, JavaScriptLockfile, PackageSource, Place};

const VERSION_9: &str = "lockfileVersion: '9.0'
importers:
  .:
    dependencies:
      runner:
        specifier: ^1.0.0
        version: 1.0.0(react@18.2.0)
      helper:
        specifier: npm:real-helper@^2.0.0
        version: real-helper@2.1.0
  apps/web:
    dependencies:
      tools:
        specifier: workspace:*
        version: link:../../packages/tools
packages:
  runner@1.0.0:
    resolution: {integrity: sha512-AAAA}
  real-helper@2.1.0:
    resolution: {integrity: sha512-BBBB}
  forked@1.0.0:
    resolution: {commit: 0123456789abcdef, repo: https://github.com/acme/forked.git, type: git}
snapshots:
  runner@1.0.0(react@18.2.0):
    dependencies:
      forked: 1.0.0
  real-helper@2.1.0: {}
  forked@1.0.0: {}
";

const VERSION_6: &str = "lockfileVersion: '6.0'
dependencies:
  runner:
    specifier: ^1.0.0
    version: 1.0.0
packages:
  /runner@1.0.0:
    resolution: {integrity: sha512-CCCC}
    dependencies:
      '@scope/inner': 3.0.0
  /@scope/inner@3.0.0:
    resolution: {integrity: sha512-DDDD}
";

const VERSION_5: &str = "lockfileVersion: 5.4
dependencies:
  runner: 1.0.0_react@18.2.0
packages:
  /runner/1.0.0_react@18.2.0:
    resolution: {integrity: sha512-EEEE}
";

fn pnpm(text: &str) -> JavaScriptLockfile {
    JavaScriptLockfile::parse("pnpm-lock.yaml", text).expect("lockfile")
}

fn registry(name: &str, version: &str, integrity: &str) -> PackageSource {
    let base_name = name.rsplit('/').next().unwrap_or(name);
    PackageSource::Registry {
        url: format!("https://registry.npmjs.org/{name}/-/{base_name}-{version}.tgz"),
        integrity: Some(integrity.to_string()),
    }
}

fn found(
    lockfile: &JavaScriptLockfile,
    place: Place,
    name: &str,
) -> (String, String, PackageSource) {
    let Found { key, package } = lockfile.find(place, name).expect("found");
    (
        key,
        format!("{}@{}", package.name, package.version),
        package.source,
    )
}

#[test]
fn version_9_reads_peers_aliases_links_and_git() {
    let lockfile = pnpm(VERSION_9);
    let root = Place::Repository { folder: "src" };

    let runner = found(&lockfile, root, "runner");
    let helper = found(&lockfile, root, "helper");
    let tools = found(
        &lockfile,
        Place::Repository {
            folder: "apps/web/src",
        },
        "tools",
    );
    let forked = found(&lockfile, Place::Package { key: &runner.0 }, "forked");

    assert_eq!(
        runner,
        (
            "runner@1.0.0(react@18.2.0)".to_string(),
            "runner@1.0.0".to_string(),
            registry("runner", "1.0.0", "sha512-AAAA")
        )
    );
    assert_eq!(helper.1, "real-helper@2.1.0");
    assert_eq!(helper.2, registry("real-helper", "2.1.0", "sha512-BBBB"));
    assert_eq!(tools.2, PackageSource::Folder("packages/tools".to_string()));
    assert_eq!(
        forked.2,
        PackageSource::Git("https://github.com/acme/forked.git#0123456789abcdef".to_string())
    );
}

#[test]
fn version_6_reads_root_dependencies_and_scoped_packages() {
    let lockfile = pnpm(VERSION_6);

    let runner = found(&lockfile, Place::Repository { folder: "" }, "runner");
    let inner = found(&lockfile, Place::Package { key: &runner.0 }, "@scope/inner");

    assert_eq!(runner.0, "/runner@1.0.0");
    assert_eq!(inner.1, "@scope/inner@3.0.0");
    assert_eq!(inner.2, registry("@scope/inner", "3.0.0", "sha512-DDDD"));
}

#[test]
fn version_5_reads_slash_keys_with_peer_suffixes() {
    let lockfile = pnpm(VERSION_5);

    let runner = found(&lockfile, Place::Repository { folder: "lib" }, "runner");

    assert_eq!(runner.1, "runner@1.0.0");
    assert_eq!(runner.2, registry("runner", "1.0.0", "sha512-EEEE"));
}

#[test]
fn a_workspace_folder_also_finds_the_dependencies_of_enclosing_importers() {
    let lockfile = pnpm(VERSION_9);

    let runner = found(
        &lockfile,
        Place::Repository {
            folder: "apps/web/src",
        },
        "runner",
    );
    let missing = lockfile.find(Place::Repository { folder: "apps/web" }, "absent");

    assert_eq!(runner.1, "runner@1.0.0");
    assert_eq!(missing, None);
}
