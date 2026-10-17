use std::fs;

use crate::support::TestRepository;

const NAME_BASED: &str = "[engines]\npython = \"deep\"\n";
const FIXED_DATE: &str =
    "import subprocess\n\n\ndef export(request):\n    return subprocess.run([\"date\"])\n";
const SHELL_WITH_INPUT: &str = "import subprocess\n\n\ndef export(request):\n    return subprocess.run(f\"date {request.args['day']}\", shell=True)\n";
const IDENTITY: &str = "exec:app.export:subprocess.run:untrusted";

fn repository_with_shell_input() -> TestRepository {
    let repository = TestRepository::new();
    let stamp = "\n\ndef stamp(day):\n    return subprocess.run([\"date\", day])\n";
    repository.write(&[
        ("vigilante.toml", NAME_BASED),
        ("app.py", &format!("{SHELL_WITH_INPUT}{stamp}")),
    ]);
    repository.commit("code");
    repository
}

#[test]
fn the_lock_command_approves_a_blocking_risk_so_check_passes() {
    let repository = repository_with_shell_input();
    let before = repository.vigilante(&["check"]);
    assert_eq!(before.code, 1);
    assert!(
        before
            .stdout
            .contains("## Vigilante: vigilante.lock needs an update")
            && before.stdout.contains("### Risks\n"),
        "{}",
        before.stdout
    );

    let lock = repository.vigilante(&["lock"]);
    let written =
        fs::read_to_string(repository.directory.path().join("vigilante.lock")).expect("lock");

    assert_eq!(lock.code, 0, "{}", lock.stderr);
    assert!(
        written.contains(&format!("risks = [\"{IDENTITY}\"]")),
        "{written}"
    );
    let review = repository.check();
    assert_eq!(review.risks.len(), 1);
    assert!(review.risks[0].approved);
    assert!(review.stale_risks.is_empty());
    assert!(review.passed());
}

#[test]
fn an_approval_for_a_risk_that_is_gone_is_stale() {
    let repository = TestRepository::new();
    repository.write(&[
        ("vigilante.toml", NAME_BASED),
        ("app.py", FIXED_DATE),
        (
            "vigilante.lock",
            &format!(
                "version = 1\n\n[code.\".\"]\npowers = [\"exec\"]\nrisks = [\"{IDENTITY}\"]\n"
            ),
        ),
    ]);
    repository.commit("code");

    let review = repository.check();

    assert_eq!(review.stale_risks.len(), 1);
    assert_eq!(review.stale_risks[0].identity, IDENTITY);
    assert!(!review.passed());
    let run = repository.vigilante(&["check"]);
    assert!(
        run.stdout
            .contains("## Vigilante: vigilante.lock needs an update")
            && run.stdout.contains("1 lock entry is stale."),
        "{}",
        run.stdout
    );
}

#[test]
fn the_reports_show_a_risk_that_is_not_approved() {
    let repository = TestRepository::new();
    repository.write(&[("vigilante.toml", NAME_BASED), ("app.py", FIXED_DATE)]);
    let base = repository.commit("base");
    repository.write(&[("app.py", SHELL_WITH_INPUT)]);
    let head = repository.commit("change");

    let markdown = repository.vigilante(&["diff", "--base", &base, "--head", &head]);
    let sarif = repository.vigilante(&[
        "diff", "--base", &base, "--head", &head, "--format", "sarif",
    ]);

    assert_eq!(markdown.code, 1);
    assert!(
        markdown.stdout.contains("### Changed risks")
            && markdown
                .stdout
                .contains("- `.` `exec` with untrusted input **not approved**: `app.py:5"),
        "{}",
        markdown.stdout
    );
    assert!(
        markdown
            .stdout
            .contains("1 risk is not approved in vigilante.lock."),
        "{}",
        markdown.stdout
    );
    let sarif: serde_json::Value = serde_json::from_str(&sarif.stdout).expect("sarif");
    let result = &sarif["runs"][0]["results"][0];
    assert_eq!(result["level"], "error");
    assert_eq!(
        result["locations"][0]["physicalLocation"]["region"]["startLine"],
        5
    );
    assert!(
        result["message"]["text"]
            .as_str()
            .is_some_and(|text| text.ends_with(IDENTITY)),
        "{result}"
    );
}
