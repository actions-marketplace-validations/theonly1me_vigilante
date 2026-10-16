use serde_json::Value;
use vigilante::Review;

use crate::support::{Branches, TestRepository};

const DEEP_PYTHON: &str = "[engines]\npython = \"deep\"\n";
const BLOCK_GAPS: &str = "[engines]\npython = \"deep\"\n[policy]\ncoverage = \"block\"\n";
const UNLOCKED_IMPORT: &str = "import unlisted_client\nunlisted_client.upload()\n";
const NO_LOCKFILE: &str = "No uv.lock, poetry.lock, Pipfile.lock, or requirements.txt covers this code, so Vigilante did not read its packages.";

struct Change<'a> {
    base: &'a [(&'a str, &'a str)],
    head: &'a [(&'a str, &'a str)],
}

fn repository_with(change: Change) -> (TestRepository, String, String) {
    let repository = TestRepository::new();
    repository.write(change.base);
    let base = repository.commit("base");
    repository.write(change.head);
    let head = repository.commit("change");
    (repository, base, head)
}

fn review_of(change: Change) -> Review {
    let (repository, base, head) = repository_with(change);
    repository.diff(Branches {
        base: &base,
        head: &head,
    })
}

fn gap_reasons(review: &Review) -> Vec<String> {
    review
        .new_gaps
        .iter()
        .map(|gap| gap.reason.clone())
        .collect()
}

#[test]
fn a_new_analysis_gap_fails_when_the_policy_blocks_gaps() {
    let review = review_of(Change {
        base: &[
            ("vigilante.toml", BLOCK_GAPS),
            ("README.md", "Application\n"),
        ],
        head: &[("app.py", UNLOCKED_IMPORT)],
    });

    assert_eq!(gap_reasons(&review), vec![NO_LOCKFILE]);
    assert!(!review.passed());
}

#[test]
fn a_new_analysis_gap_only_warns_by_default() {
    let review = review_of(Change {
        base: &[
            ("vigilante.toml", DEEP_PYTHON),
            ("README.md", "Application\n"),
        ],
        head: &[("app.py", UNLOCKED_IMPORT)],
    });

    assert_eq!(gap_reasons(&review), vec![NO_LOCKFILE]);
    assert!(review.passed());
}

#[test]
fn a_gap_that_the_base_already_had_is_not_new() {
    let review = review_of(Change {
        base: &[("vigilante.toml", BLOCK_GAPS), ("app.py", UNLOCKED_IMPORT)],
        head: &[(
            "app.py",
            "import unlisted_client\nunlisted_client.upload()\nunlisted_client.close()\n",
        )],
    });

    assert_eq!(gap_reasons(&review), Vec::<String>::new());
    assert!(review.passed());
}

#[test]
fn the_sarif_report_lists_each_analysis_gap() {
    let (repository, base, head) = repository_with(Change {
        base: &[
            ("vigilante.toml", DEEP_PYTHON),
            ("README.md", "Application\n"),
        ],
        head: &[("app.py", UNLOCKED_IMPORT)],
    });

    let run = repository.vigilante(&[
        "diff", "--base", &base, "--head", &head, "--format", "sarif",
    ]);

    let sarif: Value = serde_json::from_str(&run.stdout).expect("sarif");
    let notifications = &sarif["runs"][0]["invocations"][0]["toolExecutionNotifications"];
    assert_eq!(
        notifications[0]["message"]["text"],
        format!(".: {NO_LOCKFILE}")
    );
    assert_eq!(notifications[0]["level"], "warning");
}

#[test]
fn the_scan_json_lists_each_analysis_gap() {
    let (repository, _, _) = repository_with(Change {
        base: &[
            ("vigilante.toml", DEEP_PYTHON),
            ("README.md", "Application\n"),
        ],
        head: &[("app.py", UNLOCKED_IMPORT)],
    });

    let run = repository.vigilante(&["scan", "--format", "json"]);

    let scan: Value = serde_json::from_str(&run.stdout).expect("scan json");
    assert_eq!(scan["not_analyzed"][0]["reason"], NO_LOCKFILE);
    assert!(scan["powers"].is_array(), "{scan}");
}

#[test]
fn the_markdown_report_marks_a_new_gap_that_fails_the_check() {
    let (repository, base, head) = repository_with(Change {
        base: &[
            ("vigilante.toml", BLOCK_GAPS),
            ("README.md", "Application\n"),
        ],
        head: &[("app.py", UNLOCKED_IMPORT)],
    });

    let run = repository.vigilante(&["diff", "--base", &base, "--head", &head]);

    assert!(
        run.stdout.contains(&format!("- `.` (new): {NO_LOCKFILE}")),
        "{}",
        run.stdout
    );
    assert!(
        run.stdout
            .contains("**Result: fail.** 1 new source is not analyzed."),
        "{}",
        run.stdout
    );
    assert_eq!(run.code, 1);
}
