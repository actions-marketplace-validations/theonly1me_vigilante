use serde_json::Value;
use vigilante::{Coverage, Format, Lock, PolicyMode, Review, ReviewMode, Skip};

use crate::support::{Branches, TestRepository};

const MAIN_AND_FEATURE: Branches = Branches {
    base: "main",
    head: "feature",
};

const PYTHON_PATTERNS: &str = "[engines]\npython = \"patterns\"\n";
const HOSTILE_SERVER: &str =
    r#"{"mcpServers": {"x](https://evil.example) @org/team | y": {"command": "npx"}}}"#;

fn feature_review(files: &[(&str, &str)]) -> Review {
    let repository = TestRepository::new();
    repository.write(&[("README.md", "# app\n")]);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(files);
    repository.commit("feature");
    repository.diff(MAIN_AND_FEATURE)
}

fn unescaped_pipes(line: &str) -> usize {
    line.match_indices('|')
        .filter(|(index, _)| !line[..*index].ends_with('\\'))
        .count()
}

#[test]
fn markdown_lists_each_new_power_and_the_fix() {
    let review = feature_review(&[
        (
            "services/billing/export.py",
            "import subprocess; subprocess.run(['true'])\n",
        ),
        (".mcp.json", HOSTILE_SERVER),
    ]);

    let markdown = review.render(Format::Markdown);

    assert!(markdown.starts_with("## Vigilante: this pull request adds 2 new powers\n"));
    assert!(markdown.contains(
        "| HIGH | `services` (code) | `exec` | Can run other programs | `services/billing/export.py:1 via export.init → subprocess.run` | **not in lock** |"
    ));
    assert_eq!(
        markdown.lines().nth(2),
        Some(
            "**Result: fail.** 2 powers are not in vigilante.lock. Run `vigilante lock` and commit the file."
        )
    );
}

#[test]
fn markdown_keeps_hostile_names_inside_one_table_cell() {
    let review = feature_review(&[(".mcp.json", HOSTILE_SERVER)]);

    let markdown = review.render(Format::Markdown);

    let row = markdown
        .lines()
        .find(|line| line.contains("mcp_server"))
        .expect("MCP row");
    assert_eq!(unescaped_pipes(row), 7, "{row}");
    assert!(
        row.contains("| `mcp_server:x](https://evil.example) @org/team \\| y: npx` |"),
        "{row}"
    );
}

#[test]
fn json_and_sarif_mark_the_blocking_finding() {
    let review = feature_review(&[(
        "services/billing/export.py",
        "import subprocess; subprocess.run(['true'])\n",
    )]);

    let json: Value = serde_json::from_str(&review.render(Format::Json)).expect("JSON");
    let sarif: Value = serde_json::from_str(&review.render(Format::Sarif)).expect("SARIF");

    assert_eq!(json["passed"], false);
    assert_eq!(json["mode"]["kind"], "diff");
    assert_eq!(json["findings"][0]["power"], "exec");
    assert_eq!(sarif["version"], "2.1.0");
    let result = &sarif["runs"][0]["results"][0];
    assert_eq!(result["ruleId"], "exec");
    assert_eq!(result["level"], "error");
    let location = &result["locations"][0]["physicalLocation"];
    assert_eq!(
        location["artifactLocation"]["uri"],
        "services/billing/export.py"
    );
    assert_eq!(location["region"]["startLine"], 1);
}

#[test]
fn a_hidden_power_is_shown_and_fails_until_the_lock_has_it() {
    let writes_a_file = "with open('out.csv', 'w') as out:\n    out.write('x')\n";
    let repository = TestRepository::new();
    repository.write(&[
        ("README.md", "# app\n"),
        ("vigilante.toml", PYTHON_PATTERNS),
    ]);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(&[("services/report.py", writes_a_file)]);
    repository.commit("write a report");

    let unlocked = repository.diff(MAIN_AND_FEATURE);

    assert!(unlocked.render(Format::Markdown).contains("| `files` |"));
    assert!(!unlocked.passed());

    repository.write(&[(
        Lock::FILE,
        "version = 1\n\n[code.services]\npowers = [\"files\"]\n",
    )]);
    repository.commit("lock the new power");

    let locked = repository.diff(MAIN_AND_FEATURE);

    let markdown = locked.render(Format::Markdown);
    assert!(!markdown.contains("| `files` |"), "{markdown}");
    assert!(
        markdown.contains("Hidden low-signal findings: 1 (files)."),
        "{markdown}"
    );
    assert!(locked.passed());
}

#[test]
fn markdown_shows_the_first_ten_lines_that_were_not_analyzed_and_counts_the_rest() {
    let review = Review {
        mode: ReviewMode::Check,
        policy: PolicyMode::Lock,
        findings: Vec::new(),
        notes: Vec::new(),
        stale: Vec::new(),
        skipped: (0..12)
            .map(|index| Skip {
                source: format!("examples/{index:02}/requirements.txt"),
                reason: "1 package has no exact version.".to_string(),
            })
            .collect(),
        coverage: Coverage::Warn,
        new_gaps: Vec::new(),
        tests: Vec::new(),
        risks: Vec::new(),
        stale_risks: Vec::new(),
    };

    let markdown = review.render(Format::Markdown);

    assert_eq!(
        markdown.matches("1 package has no exact version.").count(),
        10,
        "{markdown}"
    );
    assert!(
        markdown.contains("2 more lines are in the JSON report."),
        "{markdown}"
    );
}
