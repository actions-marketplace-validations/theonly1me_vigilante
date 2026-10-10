use std::collections::HashMap;

use super::{Declared, Query, Tarball, found, from_repository, registry, yarn};
use crate::lockfiles::{Lookup, PackageSource, Place};

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

#[test]
fn version_1_reads_declared_ranges_dependencies_aliases_and_sha1_hashes() {
    let lockfile = yarn(CLASSIC);

    let runner = found(Query {
        lockfile: &lockfile,
        lookup: from_repository(Declared {
            name: "runner",
            declared: Some("^1.0.0"),
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
    let alias = found(Query {
        lockfile: &lockfile,
        lookup: from_repository(Declared {
            name: "string-width-cjs",
            declared: Some("npm:string-width@^4.2.0"),
        }),
    });

    assert_eq!(
        runner.2,
        registry(Tarball {
            url: "https://registry.yarnpkg.com/runner/-/runner-1.0.0.tgz",
            integrity: Some("sha1-ASNFZ4mrze8BI0VniavN7wEjRWc=")
        })
    );
    assert_eq!(spawner.1, "spawner@2.0.0");
    assert_eq!(alias.1, "string-width@4.2.3");
    assert_eq!(
        lockfile.find(from_repository(Declared {
            name: "spawner",
            declared: None
        })),
        None
    );
}

#[test]
fn version_1_finds_workspace_packages_by_name() {
    let mut lockfile = yarn(CLASSIC);
    lockfile.add_workspaces(HashMap::from([(
        "@acme/tools".to_string(),
        "packages/tools".to_string(),
    )]));

    let tools = found(Query {
        lockfile: &lockfile,
        lookup: from_repository(Declared {
            name: "@acme/tools",
            declared: Some("*"),
        }),
    });

    assert_eq!(tools.2, PackageSource::Folder("packages/tools".to_string()));
}
