use vigilante::{Format, Lock, ScanRequest, Source};

use crate::risks::{Change, FIXED_DATE, review_of};
use crate::support::{Branches, TestRepository};

const MAIN_AND_FEATURE: Branches = Branches {
    base: "main",
    head: "feature",
};

fn top_lines(markdown: &str) -> Vec<&str> {
    markdown.lines().take(3).collect()
}

#[test]
fn the_result_and_the_action_come_first_when_only_a_risk_changes() {
    let review = review_of(Change {
        base: &[("app.py", FIXED_DATE)],
        head: &[(
            "app.py",
            "import subprocess\n\n\ndef export(request):\n    return subprocess.run(request.args[\"command\"], shell=True)\n",
        )],
    });

    let markdown = review.render(Format::Markdown);

    assert_eq!(
        top_lines(&markdown),
        vec![
            "## Vigilante: this pull request changes 1 risk",
            "",
            "**Result: fail.** 1 risk is not approved in vigilante.lock. Review it, then run `vigilante lock` to approve it.",
        ]
    );
}

#[test]
fn a_new_power_in_the_lock_asks_for_a_code_owner_review() {
    let repository = TestRepository::new();
    repository.write(&[("README.md", "# app\n")]);
    let base = repository.commit("base");
    repository.write(&[(
        ".github/workflows/release.yml",
        "on: push\npermissions:\n  contents: write\njobs: {}\n",
    )]);
    repository.vigilante(&["lock"]);
    let head = repository.commit("add a workflow that can push, and lock it");

    let markdown = repository
        .diff(Branches {
            base: &base,
            head: &head,
        })
        .render(Format::Markdown);

    assert_eq!(
        top_lines(&markdown),
        vec![
            "## Vigilante: this pull request adds 1 new power",
            "",
            "**Result: pass.** 1 new power is in vigilante.lock. A code owner of vigilante.lock must approve this pull request.",
        ]
    );
    assert!(markdown.ends_with("| in lock |\n"), "{markdown}");
}

#[test]
fn an_approved_risk_asks_for_a_code_owner_review() {
    let review = review_of(Change {
        base: &[("app.py", FIXED_DATE)],
        head: &[
            (
                "app.py",
                "import subprocess\n\n\ndef export(request):\n    return subprocess.run(request.args[\"command\"], shell=True)\n",
            ),
            (
                "vigilante.lock",
                "version = 1\n\n[code.\".\"]\npowers = [\"exec\"]\nrisks = [\"exec:app.export:subprocess.run:untrusted\"]\n",
            ),
        ],
    });

    let markdown = review.render(Format::Markdown);

    assert_eq!(
        markdown.lines().nth(2),
        Some(
            "**Result: pass.** 1 risk is in vigilante.lock. A code owner of vigilante.lock must approve this pull request."
        ),
        "{markdown}"
    );
}

#[test]
fn markdown_hides_locked_system_state_reads_but_counts_them() {
    let repository = TestRepository::new();
    repository.write(&[
        ("README.md", "# app\n"),
        ("vigilante.toml", "[engines]\npython = \"patterns\"\n"),
    ]);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(&[(
        "services/run.py",
        "import os\nimport subprocess\nvalue = os.environ['A']\n",
    )]);
    let map = vigilante::scan(ScanRequest {
        repository: repository.directory.path().to_path_buf(),
        source: Source::WorkingDirectory,
    })
    .expect("scan");
    repository.write(&[(Lock::FILE, &Lock::of(&map).render())]);
    repository.commit("feature");

    let markdown = repository.diff(MAIN_AND_FEATURE).render(Format::Markdown);

    assert!(markdown.contains("| `exec` |"), "{markdown}");
    assert!(!markdown.contains("| `read_system_state` |"), "{markdown}");
    assert!(
        markdown.contains(
            "Hidden low-signal findings: 1 (read_system_state). They stay in vigilante.lock."
        ),
        "{markdown}"
    );
    assert_eq!(
        markdown.lines().nth(2),
        Some(
            "**Result: pass.** 2 new powers are in vigilante.lock. A code owner of vigilante.lock must approve this pull request."
        ),
        "{markdown}"
    );
}
