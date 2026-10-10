use std::collections::BTreeSet;

use serde_json::Value;

use crate::go_fixtures::{
    APP_MODULE, APP_MODULE_FILE, Call, DIFF, EXEC_COMMAND, FETCH_PACKAGE, FETCH_SOURCE, PAGE,
    Reach, capslock_output, exec_reaches, module_repository, network_reaches,
};
use crate::go_support::{FakeTools, GoRun, Tools, report_of, run_vigilante};
use crate::support::{Run, TestRepository};

const DIRECT_EXEC: Call = Call {
    function: "os/exec.Command",
    package: "os/exec",
    site: Some(("fetch.go", 4)),
};
const MARKDOWN_DIFF: &[&str] = &["diff", "--base", "main", "--head", "feature"];

fn diff(repository: &TestRepository, arguments: &[&str]) -> Run {
    let tools = FakeTools::new();
    run_vigilante(GoRun {
        repository,
        tools: &tools,
        available: Tools::Fake,
        arguments,
        environment: &[],
    })
}

fn note_summaries(report: &Value) -> BTreeSet<String> {
    report["notes"]
        .as_array()
        .expect("notes")
        .iter()
        .map(|note| {
            format!(
                "{} {} via {} {}:{}",
                note["scope"]["path"].as_str().unwrap_or_default(),
                note["power"].as_str().unwrap_or_default(),
                note["through"].as_str().unwrap_or_default(),
                note["evidence"]["path"].as_str().unwrap_or_default(),
                note["evidence"]["line"]
            )
        })
        .collect()
}

fn existing_exec_reaches_a_new_dependency() -> TestRepository {
    let direct: Vec<Reach> = [FETCH_PACKAGE, "os/exec"]
        .into_iter()
        .map(|package| Reach {
            package,
            capability: "EXEC",
            calls: &[PAGE, DIRECT_EXEC],
        })
        .collect();
    let repository = TestRepository::new();
    repository.write(&[
        (APP_MODULE, APP_MODULE_FILE),
        ("app/fetch/fetch.go", FETCH_SOURCE),
        ("app/capslock.json", &capslock_output(&direct)),
        (
            "vigilante.lock",
            "version = 1\n\n[code.\"app/fetch\"]\npowers = [\"exec\"]\n",
        ),
    ]);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    let mut through_dependency = exec_reaches();
    through_dependency.push(Reach {
        package: "example.com/dep/run",
        capability: "FILES",
        calls: &[PAGE, EXEC_COMMAND],
    });
    repository.write(&[("app/capslock.json", &capslock_output(&through_dependency))]);
    repository.commit("run the tool through a dependency");
    repository
}

#[test]
fn diff_notes_each_new_package_on_a_call_path() {
    let repository = module_repository(&network_reaches());
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    let both: Vec<Reach> = network_reaches()
        .into_iter()
        .chain(exec_reaches())
        .collect();
    repository.write(&[("app/capslock.json", &capslock_output(&both))]);
    repository.commit("run a tool");

    let report = report_of(&diff(&repository, DIFF));

    let expected: BTreeSet<String> = [
        "app/fetch exec via example.com/dep/run app/fetch/fetch.go:6",
        "app/fetch exec via os/exec app/fetch/fetch.go:6",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert_eq!(note_summaries(&report), expected);
}

#[test]
fn a_new_path_to_a_locked_power_is_a_note_and_the_check_passes() {
    let repository = existing_exec_reaches_a_new_dependency();

    let run = diff(&repository, MARKDOWN_DIFF);

    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stdout.contains(
            "| `app/fetch` (code) | `exec` | `example.com/dep/run` | `app/fetch/fetch.go:6 via fetch.Page → run.Tool → exec.Command` |"
        ),
        "{}",
        run.stdout
    );
    assert!(!run.stdout.contains("| `files` |"), "{}", run.stdout);
    assert!(
        run.stdout.contains(
            "Hidden notes for low-signal powers: 1 (files). They are in the JSON report."
        ),
        "{}",
        run.stdout
    );
}

#[test]
fn sarif_and_json_carry_the_call_path() {
    let repository = module_repository(&network_reaches());
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    let both: Vec<Reach> = network_reaches()
        .into_iter()
        .chain(exec_reaches())
        .collect();
    repository.write(&[("app/capslock.json", &capslock_output(&both))]);
    repository.commit("run a tool");

    let json = report_of(&diff(&repository, DIFF));
    let sarif = report_of(&diff(
        &repository,
        &[
            "diff", "--base", "main", "--head", "feature", "--format", "sarif",
        ],
    ));

    assert_eq!(
        json["findings"][0]["evidence"]["call_path"],
        serde_json::json!([
            "example.com/app/fetch.Page",
            "example.com/dep/run.Tool",
            "os/exec.Command"
        ])
    );
    let message = sarif["runs"][0]["results"][0]["message"]["text"].to_string();
    assert!(
        message.contains("fetch.Page → run.Tool → exec.Command"),
        "{message}"
    );
}
