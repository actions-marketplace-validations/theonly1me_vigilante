mod fixture;
mod forms;

use crate::lockfiles::{Found, JavaScriptLockfile, LockfileText, Lookup, PackageSource, Place};
use fixture::BUN_1_4_LOCKFILE;

const ROOT: Place = Place::Repository { folder: "" };
const IS_NUMBER_7: &str = "sha512-41Cifkg6e8TylSpdtTpeLVMqvSBEVzTttHvERD741+pnZ8ANv0004MRL43QKPDlK9cGvNp6NZWZUBlbGXYxxng==";
const IS_NUMBER_6: &str = "sha512-Wu1VHeILBK8KAWJUAiSZQX94GmOE45Rg6/538fKwiloUu21KncEkYGPqob2oSZ5mUT73vLGrHQjKw3KMPwfDzg==";

fn bun(text: &str) -> JavaScriptLockfile {
    JavaScriptLockfile::parse(LockfileText {
        path: "bun.lock",
        text,
    })
    .expect("lockfile")
}

struct Query<'a> {
    lockfile: &'a JavaScriptLockfile,
    place: Place<'a>,
    name: &'a str,
}

fn found(query: Query) -> (String, String, PackageSource) {
    let Query {
        lockfile,
        place,
        name,
    } = query;
    let Found { key, package } = lockfile
        .find(Lookup {
            place,
            name,
            declared: None,
        })
        .expect("found");
    (
        key,
        format!("{}@{}", package.name, package.version),
        package.source,
    )
}

fn npmjs(path: &str, integrity: &str) -> PackageSource {
    PackageSource::Registry {
        url: format!("https://registry.npmjs.org/{path}"),
        integrity: Some(integrity.to_string()),
    }
}

#[test]
fn a_root_dependency_comes_from_npmjs_with_its_integrity() {
    let lockfile = bun(BUN_1_4_LOCKFILE);

    assert_eq!(
        found(Query {
            lockfile: &lockfile,
            place: ROOT,
            name: "is-number",
        }),
        (
            "is-number".to_string(),
            "is-number@7.0.0".to_string(),
            npmjs("is-number/-/is-number-7.0.0.tgz", IS_NUMBER_7)
        )
    );
    assert_eq!(
        found(Query {
            lockfile: &lockfile,
            place: ROOT,
            name: "@sindresorhus/is",
        })
        .2,
        npmjs(
            "@sindresorhus/is/-/is-4.6.0.tgz",
            "sha512-t09vSN3MdfsyCHoFcTRCH/iUtG7OJ0CsjzB8cjAmKc/va/kIgeDI/TxsigdncE/4be734m0cvIYwNaV4i2XqAw=="
        )
    );
}

#[test]
fn an_alias_reads_the_real_package_name() {
    let lockfile = bun(BUN_1_4_LOCKFILE);

    let (key, label, _) = found(Query {
        lockfile: &lockfile,
        place: ROOT,
        name: "my-alias",
    });

    assert_eq!((key.as_str(), label.as_str()), ("my-alias", "is-odd@3.0.1"));
}

#[test]
fn a_package_reads_its_nested_version_before_the_hoisted_one() {
    let lockfile = bun(BUN_1_4_LOCKFILE);

    assert_eq!(
        found(Query {
            lockfile: &lockfile,
            place: Place::Package { key: "my-alias" },
            name: "is-number",
        }),
        (
            "my-alias/is-number".to_string(),
            "is-number@6.0.0".to_string(),
            npmjs("is-number/-/is-number-6.0.0.tgz", IS_NUMBER_6)
        )
    );
    assert_eq!(
        found(Query {
            lockfile: &lockfile,
            place: Place::Package {
                key: "is-odd/is-number/kind-of"
            },
            name: "is-buffer",
        })
        .1,
        "is-buffer@1.1.6"
    );
}

#[test]
fn code_in_a_workspace_reads_the_versions_under_the_workspace_name() {
    let lockfile = bun(BUN_1_4_LOCKFILE);
    let from_app = |name| Query {
        lockfile: &lockfile,
        place: Place::Repository {
            folder: "packages/app/src",
        },
        name,
    };

    assert_eq!(found(from_app("is-number")).0, "app/is-number");
    assert_eq!(
        found(from_app("shared")).2,
        PackageSource::Folder("packages/shared".to_string())
    );
}

#[test]
fn code_in_a_folder_dependency_reads_the_versions_under_its_key() {
    let lockfile = bun(BUN_1_4_LOCKFILE);

    assert_eq!(
        found(Query {
            lockfile: &lockfile,
            place: ROOT,
            name: "local-tool",
        })
        .2,
        PackageSource::Folder("tools/local-tool".to_string())
    );
    assert_eq!(
        found(Query {
            lockfile: &lockfile,
            place: Place::Repository {
                folder: "tools/local-tool"
            },
            name: "is-number",
        })
        .0,
        "local-tool/is-number"
    );
}

#[test]
fn git_and_tarball_dependencies_keep_their_sources() {
    let lockfile = bun(BUN_1_4_LOCKFILE);
    let tarball = "https://registry.npmjs.org/is-even/-/is-even-1.0.0.tgz";

    assert_eq!(
        found(Query {
            lockfile: &lockfile,
            place: ROOT,
            name: "kind-of",
        })
        .2,
        PackageSource::Git("github:jonschlinkert/kind-of#3c7e8fd".to_string())
    );
    assert_eq!(
        found(Query {
            lockfile: &lockfile,
            place: ROOT,
            name: "is-even",
        }),
        (
            "is-even".to_string(),
            format!("is-even@{tarball}"),
            PackageSource::Registry {
                url: tarball.to_string(),
                integrity: Some("sha512-LEhnkAdJqic4Dbqn58A0y52IXoHWlsueqQkKfMfdEnIYG8A1sm/GHidKkS6yvXlMoRrkM34csHnXQtOqcb+Jzg==".to_string()),
            }
        )
    );
}
