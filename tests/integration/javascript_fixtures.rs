use serde_json::Value;

use crate::npm_registry::{FakePackage, LockEntry, NpmRegistry, run_with_cache};
use crate::support::{Run, TestRepository};

const DIFF: &[&str] = &[
    "diff", "--base", "main", "--head", "feature", "--format", "json",
];
const MANIFEST: (&str, &str) = (
    "package.json",
    "{\"name\": \"app\", \"dependencies\": {\"runner\": \"^1.0.0\", \"ghost\": \"^1.0.0\"}}",
);
pub const UNUSED_MAIN: &str = "export function main() {\n  return 0;\n}\n";
pub const RUNNING_MAIN: &str =
    "import { start } from 'runner';\n\nexport function main() {\n  return start('ls');\n}\n";
pub const RUNNER: FakePackage = FakePackage {
    name: "runner",
    version: "1.0.0",
    files: &[
        ("index.js", "export { run as start } from './lib/run.js';\n"),
        (
            "lib/run.js",
            "const { spawnIt } = require('spawner');\n\nexports.run = function run(command) {\n  return spawnIt(command);\n};\n",
        ),
    ],
};
pub const SPAWNER: FakePackage = FakePackage {
    name: "spawner",
    version: "2.0.0",
    files: &[(
        "index.js",
        "const cp = require('child_process');\n\nexports.spawnIt = function spawnIt(command) {\n  return cp.execSync(command);\n};\n",
    )],
};
pub const HARMLESS_SPAWNER: FakePackage = FakePackage {
    name: "spawner",
    version: "1.0.0",
    files: &[(
        "index.js",
        "exports.spawnIt = function spawnIt(command) {\n  return command;\n};\n",
    )],
};

pub fn lock_entries() -> [LockEntry<'static>; 3] {
    [
        LockEntry {
            key: "node_modules/runner",
            package: &RUNNER,
        },
        LockEntry {
            key: "node_modules/runner/node_modules/spawner",
            package: &SPAWNER,
        },
        LockEntry {
            key: "node_modules/spawner",
            package: &HARMLESS_SPAWNER,
        },
    ]
}

pub struct Change<'a> {
    pub base_main: &'a str,
    pub head_main: &'a str,
    pub config: &'a str,
    pub lockfile: Option<&'a str>,
}

pub fn diff(change: Change) -> Run {
    let repository = TestRepository::new();
    repository.write(&[
        MANIFEST,
        ("vigilante.toml", change.config),
        ("src/index.js", change.base_main),
    ]);
    if let Some(lockfile) = change.lockfile {
        repository.write(&[("package-lock.json", lockfile)]);
    }
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(&[("src/index.js", change.head_main)]);
    repository.commit("change");
    run_with_cache(&repository, DIFF)
}

pub fn served() -> NpmRegistry {
    NpmRegistry::serve(&[&RUNNER, &SPAWNER, &HARMLESS_SPAWNER])
}

pub fn notes(report: &Value) -> Vec<String> {
    report["notes"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|note| {
            let call_path: Vec<&str> = note["evidence"]["call_path"]
                .as_array()
                .map(|names| names.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            format!(
                "{} {} through {} {}:{} {}",
                note["scope"]["path"].as_str().unwrap_or_default(),
                note["power"].as_str().unwrap_or_default(),
                note["through"].as_str().unwrap_or_default(),
                note["evidence"]["path"].as_str().unwrap_or_default(),
                note["evidence"]["line"],
                call_path.join(" > ")
            )
        })
        .collect()
}

pub fn skipped(report: &Value) -> Vec<String> {
    report["skipped"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|skip| {
            format!(
                "{}: {}",
                skip["source"].as_str().unwrap_or_default(),
                skip["reason"].as_str().unwrap_or_default()
            )
        })
        .collect()
}
