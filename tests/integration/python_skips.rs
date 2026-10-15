use serde_json::Value;

use crate::python_fixtures::{Change, RUNNER, SPAWNER, diff};
use crate::python_index::PythonIndex;
use crate::support::TestRepository;

const DIFF: &[&str] = &[
    "diff", "--base", "main", "--head", "feature", "--format", "json",
];
const CALLS_RUNNER: (&str, &str) = (
    "app.py",
    "from runner import begin\n\n\ndef main():\n    return begin(['ls'])\n",
);

fn reasons(report: &Value) -> Vec<String> {
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

#[test]
fn a_wheel_that_does_not_match_its_lockfile_hash_stops_the_review() {
    let index = PythonIndex::serve(&[&RUNNER, &SPAWNER]);
    let lock = index.uv_lock(&[&RUNNER, &SPAWNER]);
    let tampered = lock.replacen("hash = \"sha256:", "hash = \"sha256:00", 1);
    let repository = TestRepository::new();
    repository.write(&[
        ("pyproject.toml", "[project]\nname = \"app\"\n"),
        ("app.py", "def main():\n    return 0\n"),
        ("vigilante.toml", &index.allow_config()),
        ("uv.lock", &tampered),
    ]);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(&[CALLS_RUNNER]);
    repository.commit("call runner");

    let run = repository.run_with_cache(DIFF);

    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(
        run.stderr
            .contains("does not match the checksum in the lockfile"),
        "{}",
        run.stderr
    );
}

#[test]
fn a_wheel_from_an_address_that_is_not_allowed_is_listed() {
    let index = PythonIndex::serve(&[&RUNNER, &SPAWNER]);
    let lock = index.uv_lock(&[&RUNNER, &SPAWNER]);

    let report = diff(Change {
        base: &[("uv.lock", &lock)],
        head: &[CALLS_RUNNER],
    });

    let reasons = reasons(&report);
    assert_eq!(reasons.len(), 1, "{reasons:#?}");
    assert!(
        reasons[0].starts_with("uv.lock: `runner==1.0.0` comes from `http://127.0.0.1:"),
        "{reasons:#?}"
    );
}

#[test]
fn code_without_a_lockfile_is_listed() {
    let report = diff(Change {
        base: &[],
        head: &[CALLS_RUNNER],
    });

    assert_eq!(
        reasons(&report),
        vec![
            ".: No uv.lock, poetry.lock, Pipfile.lock, or requirements.txt covers this code, so Vigilante did not read its packages."
        ]
    );
}

#[test]
fn requirements_without_exact_versions_are_listed() {
    let report = diff(Change {
        base: &[("requirements.txt", "runner>=1.0\n")],
        head: &[CALLS_RUNNER],
    });

    let reasons = reasons(&report);
    assert!(
        reasons.contains(
            &"requirements.txt: `runner` has no exact version, so Vigilante did not read it."
                .to_string()
        ),
        "{reasons:#?}"
    );
}
