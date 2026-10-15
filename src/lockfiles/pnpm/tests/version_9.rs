use super::{Published, Query, found, pnpm, registry};
use crate::lockfiles::{Lookup, PackageSource, Place};

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

#[test]
fn version_9_reads_peers_aliases_links_and_git() {
    let lockfile = pnpm(VERSION_9);
    let root = Place::Repository { folder: "src" };

    let runner = found(Query {
        lockfile: &lockfile,
        place: root,
        name: "runner",
    });
    let helper = found(Query {
        lockfile: &lockfile,
        place: root,
        name: "helper",
    });
    let tools = found(Query {
        lockfile: &lockfile,
        place: Place::Repository {
            folder: "apps/web/src",
        },
        name: "tools",
    });
    let forked = found(Query {
        lockfile: &lockfile,
        place: Place::Package { key: &runner.0 },
        name: "forked",
    });

    assert_eq!(
        runner,
        (
            "runner@1.0.0(react@18.2.0)".to_string(),
            "runner@1.0.0".to_string(),
            registry(Published {
                name: "runner",
                version: "1.0.0",
                integrity: "sha512-AAAA"
            })
        )
    );
    assert_eq!(helper.1, "real-helper@2.1.0");
    assert_eq!(
        helper.2,
        registry(Published {
            name: "real-helper",
            version: "2.1.0",
            integrity: "sha512-BBBB"
        })
    );
    assert_eq!(tools.2, PackageSource::Folder("packages/tools".to_string()));
    assert_eq!(
        forked.2,
        PackageSource::Git("https://github.com/acme/forked.git#0123456789abcdef".to_string())
    );
}

#[test]
fn a_workspace_folder_also_finds_the_dependencies_of_enclosing_importers() {
    let lockfile = pnpm(VERSION_9);

    let runner = found(Query {
        lockfile: &lockfile,
        place: Place::Repository {
            folder: "apps/web/src",
        },
        name: "runner",
    });
    let missing = lockfile.find(Lookup {
        place: Place::Repository { folder: "apps/web" },
        name: "absent",
        declared: None,
    });

    assert_eq!(runner.1, "runner@1.0.0");
    assert_eq!(missing, None);
}
