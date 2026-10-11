use serde_json::{Value, json};

use crate::npm_registry::{FakePackage, LockEntry, NpmRegistry, report_of};
use crate::support::TestRepository;

const DIFF: &[&str] = &[
    "diff", "--base", "main", "--head", "feature", "--format", "json",
];
const WEB_MAIN: &str =
    "import { start } from 'runner';\n\nexport function main() {\n  return start('ls');\n}\n";
const RUNNER: FakePackage = FakePackage {
    name: "runner",
    version: "1.0.0",
    files: &[(
        "index.js",
        "const { spawnIt } = require('spawner');\n\nexports.start = function start(command) {\n  return spawnIt(command);\n};\n",
    )],
};
const SPAWNER: FakePackage = FakePackage {
    name: "spawner",
    version: "2.0.0",
    files: &[(
        "index.js",
        "const cp = require('child_process');\n\nexports.spawnIt = function spawnIt(command) {\n  return cp.execSync(command);\n};\n",
    )],
};
const HARMLESS_SPAWNER: FakePackage = FakePackage {
    name: "spawner",
    version: "1.0.0",
    files: &[(
        "index.js",
        "exports.spawnIt = function spawnIt(command) {\n  return command;\n};\n",
    )],
};

fn powers(list: &Value) -> Vec<String> {
    list.as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|entry| {
            format!(
                "{} {} {}",
                entry["scope"]["path"].as_str().unwrap_or_default(),
                entry["power"].as_str().unwrap_or_default(),
                entry["through"].as_str().unwrap_or("own code")
            )
        })
        .collect()
}

#[test]
fn a_lockfile_change_reviews_the_code_that_uses_it() {
    let registry = NpmRegistry::serve(&[&RUNNER, &SPAWNER, &HARMLESS_SPAWNER]);
    let lockfile_with = |spawner: &FakePackage| {
        registry.lockfile(&[
            LockEntry {
                key: "node_modules/runner",
                package: &RUNNER,
            },
            LockEntry {
                key: "node_modules/runner/node_modules/spawner",
                package: spawner,
            },
        ])
    };
    let repository = TestRepository::new();
    repository.write(&[
        (
            "package.json",
            "{\"name\": \"root\", \"workspaces\": [\"apps/*\"]}",
        ),
        ("apps/web/package.json", "{\"name\": \"web\"}"),
        ("apps/web/src/index.js", WEB_MAIN),
        ("vigilante.toml", &registry.allow_config()),
        ("package-lock.json", &lockfile_with(&HARMLESS_SPAWNER)),
    ]);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(&[("package-lock.json", &lockfile_with(&SPAWNER))]);
    repository.commit("update spawner");

    let run = repository.run_with_cache(DIFF);

    let report = report_of(&run);
    assert!(
        powers(&report["notes"]).contains(&"apps/web exec runner".to_string()),
        "{}\n{}",
        run.stdout,
        run.stderr
    );
}

#[test]
fn a_workspace_package_is_own_code() {
    let registry = NpmRegistry::serve(&[]);
    let mut lockfile: Value = serde_json::from_str(&registry.lockfile(&[])).expect("lockfile");
    lockfile["packages"]["node_modules/@acme/tools"] =
        json!({ "resolved": "packages/tools", "link": true });
    lockfile["packages"]["packages/tools"] = json!({ "name": "@acme/tools", "version": "1.0.0" });
    let repository = TestRepository::new();
    repository.write(&[
        ("package.json", "{\"name\": \"root\", \"workspaces\": [\"apps/*\", \"packages/*\"]}"),
        ("apps/web/package.json", "{\"name\": \"web\"}"),
        ("apps/web/src/index.js", "export function main() {\n  return 0;\n}\n"),
        ("packages/tools/package.json", "{\"name\": \"@acme/tools\", \"main\": \"lib/main.js\"}"),
        (
            "packages/tools/lib/main.js",
            "const cp = require('child_process');\n\nexports.build = function build() {\n  return cp.execSync('make');\n};\n",
        ),
        ("package-lock.json", &lockfile.to_string()),
    ]);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(&[(
        "apps/web/src/index.js",
        "import { build } from '@acme/tools';\n\nexport function main() {\n  return build();\n}\n",
    )]);
    repository.commit("call the tools package");

    let run = repository.run_with_cache(DIFF);

    let report = report_of(&run);
    assert_eq!(
        powers(&report["notes"]),
        vec!["apps/web exec packages/tools"],
        "{}",
        run.stderr
    );
    assert_eq!(
        report["notes"][0]["evidence"]["call_path"],
        json!([
            "apps/web/src/index.main",
            "packages/tools/lib/main.build",
            "child_process.execSync"
        ])
    );
}
