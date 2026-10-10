use crate::javascript_fixtures::{
    Change, HARMLESS_SPAWNER, RUNNER, RUNNING_MAIN, SPAWNER, UNUSED_MAIN, notes, served,
};
use crate::npm_registry::{PnpmEntry, report_of, run_with_cache};
use crate::support::TestRepository;

const DIFF: &[&str] = &[
    "diff", "--base", "main", "--head", "feature", "--format", "json",
];

#[test]
fn a_pnpm_lockfile_resolves_each_package_to_its_own_dependencies() {
    let registry = served();
    let lockfile = registry.pnpm_lockfile(&[
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
    ]);
    let change = Change {
        base_main: UNUSED_MAIN,
        head_main: RUNNING_MAIN,
        config: &registry.allow_config(),
        lockfile: None,
    };
    let repository = TestRepository::new();
    repository.write(&[
        ("package.json", "{\"name\": \"app\"}"),
        ("vigilante.toml", change.config),
        ("src/index.js", change.base_main),
        ("pnpm-lock.yaml", &lockfile),
    ]);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(&[("src/index.js", change.head_main)]);
    repository.commit("change");

    let run = run_with_cache(&repository, DIFF);

    let path =
        "src/index.main > runner/lib/run.run > spawner/index.spawnIt > child_process.execSync";
    assert_eq!(
        notes(&report_of(&run)),
        vec![
            format!(". exec through runner src/index.js:4 {path}"),
            format!(". exec through runner src/index.js:4 {path}"),
            format!(". exec through spawner src/index.js:4 {path}"),
        ],
        "{}",
        run.stderr
    );
}
