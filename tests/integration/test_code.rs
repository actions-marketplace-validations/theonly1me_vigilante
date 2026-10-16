use vigilante::{Finding, Review};

use crate::support::{Branches, TestRepository, finding_lines};

const PATTERNS: &str = "[engines]\npython = \"deep\"\njavascript = \"deep\"\n";
const SHELL_TEST: &str =
    "import subprocess\n\n\ndef test_tool():\n    subprocess.run(\"make check\", shell=True)\n";

struct Change<'a> {
    base: &'a [(&'a str, &'a str)],
    head: &'a [(&'a str, &'a str)],
}

fn review_of(change: Change) -> Review {
    let repository = TestRepository::new();
    repository.write(change.base);
    let base = repository.commit("base");
    repository.write(change.head);
    let head = repository.commit("change");
    repository.diff(Branches {
        base: &base,
        head: &head,
    })
}

fn test_lines(findings: &[Finding]) -> Vec<String> {
    findings
        .iter()
        .map(|finding| {
            format!(
                "{} {} {} {} denied={}",
                finding.scope.surface.name(),
                finding.scope.path,
                finding.power,
                finding.evidence.path,
                finding.denied
            )
        })
        .collect()
}

#[test]
fn a_new_test_that_runs_a_shell_is_reported_as_test_code() {
    let review = review_of(Change {
        base: &[("vigilante.toml", PATTERNS), ("README.md", "Tools\n")],
        head: &[("tests/test_tool.py", SHELL_TEST)],
    });

    assert_eq!(
        test_lines(&review.tests),
        vec!["test tests exec tests/test_tool.py denied=false"]
    );
    assert_eq!(finding_lines(&review), Vec::<String>::new());
    assert!(review.passed());
}

#[test]
fn a_denied_power_in_test_code_blocks() {
    let config = format!("{PATTERNS}[policy]\ndeny = [{{ capability = \"exec\" }}]\n");
    let review = review_of(Change {
        base: &[("vigilante.toml", &config), ("README.md", "Tools\n")],
        head: &[("tests/test_tool.py", SHELL_TEST)],
    });

    assert_eq!(
        test_lines(&review.tests),
        vec!["test tests exec tests/test_tool.py denied=true"]
    );
    assert!(!review.passed());
}

#[test]
fn a_test_power_that_the_base_already_had_in_the_same_file_is_not_new() {
    let review = review_of(Change {
        base: &[
            ("vigilante.toml", PATTERNS),
            ("tests/test_tool.py", SHELL_TEST),
        ],
        head: &[(
            "tests/test_tool.py",
            "import subprocess\n\n\ndef test_tool():\n    subprocess.run(\"make lint\", shell=True)\n",
        )],
    });

    assert_eq!(test_lines(&review.tests), Vec::<String>::new());
}

#[test]
fn the_lock_policy_puts_test_powers_in_the_lock() {
    let config = format!("{PATTERNS}[policy]\ntests = \"lock\"\n");
    let review = review_of(Change {
        base: &[("vigilante.toml", &config), ("README.md", "Tools\n")],
        head: &[("tests/test_tool.py", SHELL_TEST)],
    });

    assert_eq!(
        finding_lines(&review),
        vec!["test tests exec locked=false denied=false"]
    );
    assert!(!review.passed());
}

#[test]
fn vendored_code_stays_out_of_the_scan() {
    let review = review_of(Change {
        base: &[("vigilante.toml", PATTERNS), ("README.md", "Tools\n")],
        head: &[(
            "node_modules/tool/index.js",
            "require('child_process').execSync('make');\n",
        )],
    });

    assert_eq!(test_lines(&review.tests), Vec::<String>::new());
    assert_eq!(finding_lines(&review), Vec::<String>::new());
}

#[test]
fn the_reports_list_test_code_apart_from_findings() {
    let repository = TestRepository::new();
    repository.write(&[("vigilante.toml", PATTERNS), ("README.md", "Tools\n")]);
    let base = repository.commit("base");
    repository.write(&[("tests/test_tool.py", SHELL_TEST)]);
    let head = repository.commit("change");

    let markdown = repository.vigilante(&["diff", "--base", &base, "--head", &head]);
    let sarif = repository.vigilante(&[
        "diff", "--base", &base, "--head", &head, "--format", "sarif",
    ]);

    assert!(
        markdown
            .stdout
            .contains("### Test code\n\nTests and fixtures run in CI"),
        "{}",
        markdown.stdout
    );
    assert!(
        markdown
            .stdout
            .contains("- `tests` `exec`: `tests/test_tool.py:1`"),
        "{}",
        markdown.stdout
    );
    let sarif: serde_json::Value = serde_json::from_str(&sarif.stdout).expect("sarif");
    assert_eq!(sarif["runs"][0]["results"][0]["level"], "note");
    assert_eq!(
        sarif["runs"][0]["results"][0]["locations"][0]["physicalLocation"]["artifactLocation"]["uri"],
        "tests/test_tool.py"
    );
}
