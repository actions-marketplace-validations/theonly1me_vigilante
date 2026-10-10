use crate::npm_registry::report_of;
use crate::rust_fixtures::{Change, RUNNER, Registry, SPAWNER, app_manifest, diff};

const RUNNING_MAIN: &str = "fn main() {\n    runner::start();\n}\n";

fn reasons(base: &[(&str, &str)]) -> Vec<String> {
    let run = diff(Change {
        base,
        head: &[("src/main.rs", RUNNING_MAIN)],
    });
    report_of(&run)["skipped"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|skip| skip["reason"].as_str().map(str::to_string))
        .collect()
}

#[test]
fn code_without_a_lockfile_is_listed() {
    let reasons = reasons(&[("Cargo.toml", &app_manifest(&["runner"]))]);

    assert_eq!(
        reasons,
        vec!["No Cargo.lock covers this code, so Vigilante did not read its crates."]
    );
}

#[test]
fn a_crate_that_is_neither_declared_nor_locked_is_listed() {
    let registry = Registry::serve(&[&SPAWNER]);

    let reasons = reasons(&[
        ("Cargo.toml", &app_manifest(&[])),
        ("Cargo.lock", &registry.lockfile(&[&SPAWNER])),
    ]);

    assert_eq!(
        reasons,
        vec![
            "Vigilante found `runner` neither in the repository nor in Cargo.lock, so it did not read it."
        ]
    );
}

#[test]
fn a_registry_that_is_not_allowed_is_listed() {
    let registry = Registry::serve(&[&RUNNER, &SPAWNER]);

    let reasons = reasons(&[
        ("Cargo.toml", &app_manifest(&["runner"])),
        ("Cargo.lock", &registry.lockfile(&[&RUNNER, &SPAWNER])),
    ]);

    assert_eq!(
        reasons,
        vec![format!(
            "The registry `{}/index/` is not public. Add it to `[registries] allow` in vigilante.toml if it needs no credentials.",
            registry.base_url
        )]
    );
}

#[test]
fn a_binary_that_uses_its_own_library_and_modules_is_not_listed() {
    let run = diff(Change {
        base: &[
            (
                "Cargo.toml",
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\n\n[lib]\nname = \"app_core\"\n",
            ),
            (
                "Cargo.lock",
                "version = 4\n\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\n",
            ),
            ("src/lib.rs", "pub mod network;\n"),
            ("src/network.rs", "pub fn open() {}\n"),
            ("src/cli.rs", "pub fn start() {}\n"),
        ],
        head: &[(
            "src/main.rs",
            "mod cli;\nuse app_core::network::open;\nuse ghost::Thing;\n\nfn main() {\n    cli::start();\n    open();\n}\n",
        )],
    });

    let reasons: Vec<String> = report_of(&run)["skipped"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|skip| skip["reason"].as_str().map(str::to_string))
        .collect();
    assert_eq!(
        reasons,
        vec![
            "Vigilante found `ghost` neither in the repository nor in Cargo.lock, so it did not read it."
        ]
    );
}

#[test]
fn a_crate_that_its_nearest_lockfile_does_not_list_is_listed_as_not_covered() {
    let run = diff(Change {
        base: &[
            ("Cargo.toml", &app_manifest(&[])),
            (
                "Cargo.lock",
                "version = 4\n\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\n",
            ),
            (
                "extra/Cargo.toml",
                "[package]\nname = \"extra\"\nversion = \"0.1.0\"\n\n[dependencies]\nserde = \"1\"\n",
            ),
            ("extra/src/lib.rs", "pub fn save() {}\n"),
        ],
        head: &[(
            "extra/src/lib.rs",
            "pub fn save() {\n    serde::export();\n}\n",
        )],
    });

    let skipped: Vec<(String, String)> = report_of(&run)["skipped"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|skip| {
            (
                skip["source"].as_str().unwrap_or_default().to_string(),
                skip["reason"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect();
    assert_eq!(
        skipped,
        vec![(
            "extra".to_string(),
            "No Cargo.lock covers this code, so Vigilante did not read its crates.".to_string()
        )]
    );
}
