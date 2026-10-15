use super::lockfile::LockedDependency;
use super::manifest::DeclaredDependency;
use super::{read_lockfile, read_manifest};

const LOCKFILE: &str = r#"version = 4

[[package]]
name = "app"
version = "0.1.0"
dependencies = [
 "http-client",
 "rand 0.8.5",
]

[[package]]
name = "http-client"
version = "1.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5b7d1d046238990b9cf5bcde22a3fb3584ee5cf65fb2765f454ed428c7a0063d"

[[package]]
name = "rand"
version = "0.8.5"
source = "git+https://github.com/rust-random/rand?branch=master#0123456789abcdef0123456789abcdef01234567"
"#;

const MANIFEST: &str = r#"[package]
name = "app"
version = "0.1.0"

[lib]
path = "lib/app.rs"

[dependencies]
http-client = "1"
randomness = { package = "rand", version = "0.8" }

[target.'cfg(unix)'.dependencies]
nix = { version = "0.29" }

[build-dependencies]
cc = "1"
"#;

#[test]
fn a_lockfile_lists_each_package_with_its_source_checksum_and_dependencies() {
    let packages = read_lockfile(LOCKFILE).expect("lockfile");

    assert_eq!(packages.len(), 3);
    assert_eq!(packages[0].source, None);
    assert_eq!(
        packages[0].dependencies,
        vec![
            LockedDependency {
                name: "http-client".to_string(),
                version: None,
            },
            LockedDependency {
                name: "rand".to_string(),
                version: Some("0.8.5".to_string()),
            },
        ]
    );
    assert_eq!(
        packages[1].checksum.as_deref(),
        Some("5b7d1d046238990b9cf5bcde22a3fb3584ee5cf65fb2765f454ed428c7a0063d")
    );
    assert!(
        packages[2]
            .source
            .as_deref()
            .is_some_and(|source| source.starts_with("git+"))
    );
}

#[test]
fn a_manifest_maps_each_crate_name_in_code_to_its_package() {
    let manifest = read_manifest(MANIFEST).expect("manifest");

    assert_eq!(manifest.name.as_deref(), Some("app"));
    assert_eq!(manifest.library.as_deref(), Some("lib/app.rs"));
    let declared: Vec<(&str, &str)> = manifest
        .dependencies
        .iter()
        .map(|DeclaredDependency { local, package }| (local.as_str(), package.as_str()))
        .collect();
    assert_eq!(
        declared,
        vec![
            ("cc", "cc"),
            ("http_client", "http-client"),
            ("nix", "nix"),
            ("randomness", "rand"),
        ]
    );
}

#[test]
fn text_that_is_not_toml_is_an_error_for_a_lockfile_and_none_for_a_manifest() {
    assert!(read_lockfile("[[package]\n").is_err());
    assert!(read_manifest("[package\n").is_none());
}

#[test]
fn a_manifest_reads_the_library_name_and_workspace_dependencies() {
    let member = read_manifest(
        "[package]\nname = \"agent\"\n\n[lib]\nname = \"agent_lib\"\n\n[dependencies]\nterminator = { workspace = true }\n",
    )
    .expect("member");
    let workspace = read_manifest(
        "[workspace]\nmembers = [\"agent\"]\n\n[workspace.dependencies]\nterminator = { path = \"terminator\", package = \"terminator-rs\" }\n",
    )
    .expect("workspace");

    assert_eq!(member.library_name.as_deref(), Some("agent_lib"));
    assert_eq!(member.inherited, vec!["terminator".to_string()]);
    assert_eq!(
        workspace.workspace_dependencies,
        vec![DeclaredDependency {
            local: "terminator".to_string(),
            package: "terminator-rs".to_string(),
        }]
    );
}
