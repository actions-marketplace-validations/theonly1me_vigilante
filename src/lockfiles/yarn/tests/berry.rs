use super::{Declared, Query, Tarball, found, from_repository, registry, yarn};
use crate::lockfiles::{Lookup, PackageSource, Place};

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

#[test]
fn berry_reads_workspaces_dependencies_and_patched_packages() {
    let lockfile = yarn(BERRY);

    let runner = found(Query {
        lockfile: &lockfile,
        lookup: from_repository(Declared {
            name: "runner",
            declared: None,
        }),
    });
    let spawner = found(Query {
        lockfile: &lockfile,
        lookup: Lookup {
            place: Place::Package { key: &runner.0 },
            name: "spawner",
            declared: None,
        },
    });
    let typescript = found(Query {
        lockfile: &lockfile,
        lookup: from_repository(Declared {
            name: "typescript",
            declared: None,
        }),
    });
    let web = found(Query {
        lockfile: &lockfile,
        lookup: from_repository(Declared {
            name: "web",
            declared: None,
        }),
    });

    assert_eq!(runner.1, "runner@1.0.0");
    assert_eq!(
        spawner.2,
        registry(Tarball {
            url: "https://registry.npmjs.org/spawner/-/spawner-2.0.0.tgz",
            integrity: None
        })
    );
    assert_eq!(
        typescript.2,
        registry(Tarball {
            url: "https://registry.npmjs.org/typescript/-/typescript-5.0.4.tgz",
            integrity: None
        })
    );
    assert_eq!(web.2, PackageSource::Folder("apps/web".to_string()));
}
