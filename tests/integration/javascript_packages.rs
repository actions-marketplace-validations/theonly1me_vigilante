use serde_json::Value;

use crate::javascript_fixtures::{
    Change, RUNNING_MAIN, UNUSED_MAIN, diff, lock_entries, notes, served,
};
use crate::npm_registry::report_of;

#[test]
fn a_new_dependency_that_runs_programs_is_a_note_with_its_call_path() {
    let registry = served();
    let lockfile = registry.lockfile(&lock_entries());

    let run = diff(Change {
        base_main: UNUSED_MAIN,
        head_main: RUNNING_MAIN,
        config: &registry.allow_config(),
        lockfile: Some(&lockfile),
    });

    let report = report_of(&run);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(report["findings"], Value::Array(Vec::new()));
    let path =
        "src/index.main > runner/lib/run.run > spawner/index.spawnIt > child_process.execSync";
    assert_eq!(
        notes(&report),
        vec![
            format!(". exec through runner src/index.js:4 {path}"),
            format!(". exec through runner src/index.js:4 {path}"),
            format!(". exec through spawner src/index.js:4 {path}"),
        ]
    );
}

#[test]
fn a_dependency_that_the_base_already_called_adds_no_note() {
    let registry = served();
    let lockfile = registry.lockfile(&lock_entries());

    let run = diff(Change {
        base_main: RUNNING_MAIN,
        head_main: &format!("{RUNNING_MAIN}\nexport const version = 2;\n"),
        config: &registry.allow_config(),
        lockfile: Some(&lockfile),
    });

    assert_eq!(notes(&report_of(&run)), Vec::<String>::new());
}

#[test]
fn a_package_that_does_not_match_its_lockfile_checksum_stops_the_review() {
    let registry = served();
    let lockfile = registry.lockfile(&lock_entries());
    let entries: Value = serde_json::from_str(&lockfile).expect("lockfile");
    let wrong = entries["packages"]["node_modules/spawner"]["integrity"]
        .as_str()
        .expect("integrity");
    let right = entries["packages"]["node_modules/runner"]["integrity"]
        .as_str()
        .expect("integrity");
    let tampered = lockfile.replacen(right, wrong, 1);

    let run = diff(Change {
        base_main: UNUSED_MAIN,
        head_main: RUNNING_MAIN,
        config: &registry.allow_config(),
        lockfile: Some(&tampered),
    });

    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(
        run.stderr
            .contains("does not match the checksum in the lockfile"),
        "{}",
        run.stderr
    );
}

#[test]
fn a_call_into_a_package_with_input_that_changes_is_not_a_risk() {
    let registry = served();
    let lockfile = registry.lockfile(&lock_entries());

    let run = diff(Change {
        base_main: RUNNING_MAIN,
        head_main: "import { start } from 'runner';\n\nexport function main(command) {\n  return start(command);\n}\n",
        config: &registry.allow_config(),
        lockfile: Some(&lockfile),
    });

    let report = report_of(&run);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(report["risks"], Value::Array(Vec::new()), "{report}");
}
