use crate::javascript_fixtures::{
    HARMLESS_SPAWNER, RUNNER, RUNNING_MAIN, SPAWNER, UNUSED_MAIN, notes, served,
};
use crate::lockfile_writers::{PnpmEntry, YarnEntry, pnpm_lockfile, yarn_lockfile};
use crate::npm_registry::{NpmRegistry, report_of, run_with_cache};
use crate::support::{Run, TestRepository};

const DIFF: &[&str] = &[
    "diff", "--base", "main", "--head", "feature", "--format", "json",
];
const PATH: &str =
    "src/index.main > runner/lib/run.run > spawner/index.spawnIt > child_process.execSync";

fn diff_with(registry: &NpmRegistry, lockfile: (&str, &str)) -> Run {
    let repository = TestRepository::new();
    repository.write(&[
        (
            "package.json",
            "{\"name\": \"app\", \"dependencies\": {\"runner\": \"^1.0.0\", \"spawner\": \"^1.0.0\"}}",
        ),
        ("vigilante.toml", &registry.allow_config()),
        ("src/index.js", UNUSED_MAIN),
        lockfile,
    ]);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(&[("src/index.js", RUNNING_MAIN)]);
    repository.commit("change");
    run_with_cache(&repository, DIFF)
}

fn expected_notes() -> Vec<String> {
    vec![
        format!(". exec through runner src/index.js:4 {PATH}"),
        format!(". exec through runner src/index.js:4 {PATH}"),
        format!(". exec through spawner src/index.js:4 {PATH}"),
    ]
}

#[test]
fn a_pnpm_lockfile_resolves_each_package_to_its_own_dependencies() {
    let registry = served();
    let lockfile = pnpm_lockfile(
        &registry,
        &[
            PnpmEntry {
                package: &RUNNER,
                dependencies: &[("spawner", "2.0.0")],
                is_direct: true,
            },
            PnpmEntry {
                package: &SPAWNER,
                dependencies: &[],
                is_direct: false,
            },
            PnpmEntry {
                package: &HARMLESS_SPAWNER,
                dependencies: &[],
                is_direct: true,
            },
        ],
    );

    let run = diff_with(&registry, ("pnpm-lock.yaml", &lockfile));

    assert_eq!(notes(&report_of(&run)), expected_notes(), "{}", run.stderr);
}

#[test]
fn a_yarn_lockfile_resolves_each_package_to_its_own_dependencies() {
    let registry = served();
    let lockfile = yarn_lockfile(
        &registry,
        &[
            YarnEntry {
                package: &RUNNER,
                range: "^1.0.0",
                dependencies: &[("spawner", "^2.0.0")],
            },
            YarnEntry {
                package: &SPAWNER,
                range: "^2.0.0",
                dependencies: &[],
            },
            YarnEntry {
                package: &HARMLESS_SPAWNER,
                range: "^1.0.0",
                dependencies: &[],
            },
        ],
    );

    let run = diff_with(&registry, ("yarn.lock", &lockfile));

    assert_eq!(notes(&report_of(&run)), expected_notes(), "{}", run.stderr);
}
