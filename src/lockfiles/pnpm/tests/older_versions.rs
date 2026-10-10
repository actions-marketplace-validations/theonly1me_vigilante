use super::{Published, Query, found, pnpm, registry};
use crate::lockfiles::Place;

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

#[test]
fn version_6_reads_root_dependencies_and_scoped_packages() {
    let lockfile = pnpm(VERSION_6);

    let runner = found(Query {
        lockfile: &lockfile,
        place: Place::Repository { folder: "" },
        name: "runner",
    });
    let inner = found(Query {
        lockfile: &lockfile,
        place: Place::Package { key: &runner.0 },
        name: "@scope/inner",
    });

    assert_eq!(runner.0, "/runner@1.0.0");
    assert_eq!(inner.1, "@scope/inner@3.0.0");
    assert_eq!(
        inner.2,
        registry(Published {
            name: "@scope/inner",
            version: "3.0.0",
            integrity: "sha512-DDDD"
        })
    );
}

#[test]
fn version_5_reads_slash_keys_with_peer_suffixes() {
    let lockfile = pnpm(VERSION_5);

    let runner = found(Query {
        lockfile: &lockfile,
        place: Place::Repository { folder: "lib" },
        name: "runner",
    });

    assert_eq!(runner.1, "runner@1.0.0");
    assert_eq!(
        runner.2,
        registry(Published {
            name: "runner",
            version: "1.0.0",
            integrity: "sha512-EEEE"
        })
    );
}
