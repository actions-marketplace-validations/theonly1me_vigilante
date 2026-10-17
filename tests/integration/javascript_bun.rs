use crate::javascript_fixtures::{
    HARMLESS_SPAWNER, RUNNER, RUNNING_MAIN, SPAWNER, UNUSED_MAIN, notes, served,
};
use crate::javascript_pnpm::{Locked, diff_with, expected_notes};
use crate::lockfile_writers::{BunEntry, BunLock};
use crate::npm_registry::{LockEntry, NpmRegistry, report_of};
use crate::support::TestRepository;

const DIFF: &[&str] = &[
    "diff", "--base", "main", "--head", "feature", "--format", "json",
];
const WEB_PATH: &str =
    "apps/web/src/index.main > runner/lib/run.run > spawner/index.spawnIt > child_process.execSync";

fn nested_spawner_lockfile(registry: &NpmRegistry) -> String {
    registry.bun_lockfile(BunLock {
        workspaces: &[],
        entries: &[
            BunEntry {
                key: "runner",
                package: &RUNNER,
                dependencies: &[("spawner", "^2.0.0")],
            },
            BunEntry {
                key: "runner/spawner",
                package: &SPAWNER,
                dependencies: &[],
            },
            BunEntry {
                key: "spawner",
                package: &HARMLESS_SPAWNER,
                dependencies: &[],
            },
        ],
    })
}

#[test]
fn a_bun_lockfile_resolves_each_package_to_its_own_dependencies() {
    let registry = served();
    let lockfile = nested_spawner_lockfile(&registry);

    let run = diff_with(Locked {
        registry: &registry,
        lockfile: ("bun.lock", &lockfile),
    });

    assert_eq!(notes(&report_of(&run)), expected_notes(), "{}", run.stderr);
}

#[test]
fn a_bun_workspace_reads_the_versions_under_its_own_name() {
    let registry = served();
    let lockfile = registry.bun_lockfile(BunLock {
        workspaces: &[("apps/web", "web")],
        entries: &[
            BunEntry {
                key: "web/runner",
                package: &RUNNER,
                dependencies: &[("spawner", "^2.0.0")],
            },
            BunEntry {
                key: "web/spawner",
                package: &SPAWNER,
                dependencies: &[],
            },
            BunEntry {
                key: "spawner",
                package: &HARMLESS_SPAWNER,
                dependencies: &[],
            },
        ],
    });
    let repository = TestRepository::new();
    repository.write(&[
        (
            "package.json",
            "{\"name\": \"app\", \"workspaces\": [\"apps/*\"]}",
        ),
        (
            "apps/web/package.json",
            "{\"name\": \"web\", \"dependencies\": {\"runner\": \"^1.0.0\"}}",
        ),
        ("apps/web/src/index.js", UNUSED_MAIN),
        ("vigilante.toml", &registry.allow_config()),
        ("bun.lock", &lockfile),
    ]);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(&[("apps/web/src/index.js", RUNNING_MAIN)]);
    repository.commit("change");

    let run = repository.run_with_cache(DIFF);

    assert_eq!(
        notes(&report_of(&run)),
        vec![
            format!("apps/web exec through runner apps/web/src/index.js:4 {WEB_PATH}"),
            format!("apps/web exec through runner apps/web/src/index.js:4 {WEB_PATH}"),
            format!("apps/web exec through spawner apps/web/src/index.js:4 {WEB_PATH}"),
        ],
        "{}",
        run.stderr
    );
}

#[test]
fn bun_lock_is_read_before_a_package_lock_in_the_same_folder() {
    let registry = served();
    let stale_package_lock = registry.lockfile(&[
        LockEntry {
            key: "node_modules/runner",
            package: &RUNNER,
        },
        LockEntry {
            key: "node_modules/spawner",
            package: &HARMLESS_SPAWNER,
        },
    ]);
    let repository = TestRepository::new();
    repository.write(&[
        (
            "package.json",
            "{\"name\": \"app\", \"dependencies\": {\"runner\": \"^1.0.0\"}}",
        ),
        ("vigilante.toml", &registry.allow_config()),
        ("src/index.js", UNUSED_MAIN),
        ("package-lock.json", &stale_package_lock),
        ("bun.lock", &nested_spawner_lockfile(&registry)),
    ]);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(&[("src/index.js", RUNNING_MAIN)]);
    repository.commit("change");

    let run = repository.run_with_cache(DIFF);

    assert_eq!(notes(&report_of(&run)), expected_notes(), "{}", run.stderr);
}
