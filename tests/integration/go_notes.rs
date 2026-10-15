use std::collections::BTreeSet;

use serde_json::Value;

use crate::go_fixtures::{
    APP_MODULE, APP_MODULE_FILE, DIFF, DIRECT_EXEC, EXEC_COMMAND, FETCH_PACKAGE, FETCH_SOURCE,
    PAGE, Reach, capslock_output, exec_reaches, finding_summaries, module_repository,
    network_reaches,
};
use crate::go_support::{FakeTools, GoRun, Tools, report_of, run_vigilante};
use crate::support::{Run, TestRepository};

const MARKDOWN_DIFF: &[&str] = &["diff", "--base", "main", "--head", "feature"];

pub struct Diff<'a> {
    pub repository: &'a TestRepository,
    pub arguments: &'a [&'a str],
}

pub fn diff(request: Diff) -> Run {
    let Diff {
        repository,
        arguments,
    } = request;
    let tools = FakeTools::new();
    run_vigilante(GoRun {
        repository,
        tools: &tools,
        available: Tools::Fake,
        arguments,
        environment: &[],
    })
}

pub fn note_summaries(report: &Value) -> BTreeSet<String> {
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
fn a_power_reached_only_through_a_dependency_is_a_note_and_the_check_passes() {
    let repository = module_repository(&network_reaches());
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    let both: Vec<Reach> = network_reaches()
        .into_iter()
        .chain(exec_reaches())
        .collect();
    repository.write(&[("app/capslock.json", &capslock_output(&both))]);
    repository.commit("run a tool from a dependency");

    let run = diff(Diff {
        repository: &repository,
        arguments: DIFF,
    });

    let report = report_of(&run);
    assert!(finding_summaries(&report).is_empty(), "{}", run.stdout);
    assert_eq!(
        note_summaries(&report),
        BTreeSet::from(["app/fetch exec via example.com/dep/run app/fetch/fetch.go:6".to_string()])
    );
    let packages: BTreeSet<&str> = report["notes"]
        .as_array()
        .expect("notes")
        .iter()
        .filter_map(|note| note["package"].as_str())
        .collect();
    assert_eq!(
        packages,
        BTreeSet::from(["example.com/app/fetch", "example.com/dep/run", "os/exec"])
    );
    assert_eq!(run.code, 0, "{}", run.stderr);
    let markdown = diff(Diff {
        repository: &repository,
        arguments: MARKDOWN_DIFF,
    })
    .stdout;
    let row = "| `app/fetch` (code) | `exec` | `example.com/dep/run` | `app/fetch/fetch.go:6 via fetch.Page → run.Tool → exec.Command` |";
    assert_eq!(markdown.matches(row).count(), 1, "{markdown}");
}

#[test]
fn a_new_path_to_a_locked_power_is_a_note_and_the_check_passes() {
    let repository = existing_exec_reaches_a_new_dependency();

    let run = diff(Diff {
        repository: &repository,
        arguments: MARKDOWN_DIFF,
    });
    let notes: BTreeSet<String> = report_of(&diff(Diff {
        repository: &repository,
        arguments: DIFF,
    }))["notes"]
        .as_array()
        .expect("notes")
        .iter()
        .map(|note| {
            format!(
                "{} {}",
                note["package"].as_str().unwrap_or_default(),
                note["power"].as_str().unwrap_or_default()
            )
        })
        .collect();
    assert_eq!(
        notes,
        BTreeSet::from([
            "example.com/dep/run exec".to_string(),
            "example.com/dep/run files".to_string()
        ])
    );

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
