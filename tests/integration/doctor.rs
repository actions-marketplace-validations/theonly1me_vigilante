use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use tempfile::TempDir;

use crate::go_support::SYSTEM_PATH;
use crate::support::{Run, TestRepository};

const FAKE_GH: &str = r#"#!/bin/sh
folder="$(dirname "$0")"
case "$*" in
  "repo view --json nameWithOwner,defaultBranchRef")
    echo '{"nameWithOwner":"acme/app","defaultBranchRef":{"name":"main"}}' ;;
  "api repos/acme/app/rules/branches/main")
    cat "$folder/rules.json" ;;
  "api repos/acme/app/branches/main/protection")
    if [ -f "$folder/protection.json" ]; then cat "$folder/protection.json"; exit 0; fi
    echo 'gh: Branch not protected (HTTP 404)' >&2
    exit 1 ;;
  *)
    echo "unexpected: $*" >&2
    exit 1 ;;
esac
"#;
const ALL_OWNED: &str = "vigilante.lock @acme/security\nvigilante.toml @acme/security\n.vigilante/ @acme/security\n.github/ @acme/security\n";
const PROTECTED_RULES: &str = r#"[{"type":"pull_request","parameters":{"require_code_owner_review":true}},{"type":"required_status_checks","parameters":{"required_status_checks":[{"context":"vigilante"}]}}]"#;

struct Doctor<'a> {
    codeowners: &'a str,
    rules: &'a str,
    protection: Option<&'a str>,
    has_gh: bool,
}

fn run_doctor(doctor: Doctor) -> Run {
    let repository = TestRepository::new();
    repository.write(&[(".github/CODEOWNERS", doctor.codeowners)]);
    let tools = TempDir::new().expect("temporary folder");
    let gh = tools.path().join("gh");
    fs::write(&gh, FAKE_GH).expect("fake gh");
    fs::set_permissions(&gh, fs::Permissions::from_mode(0o755)).expect("executable");
    fs::write(tools.path().join("rules.json"), doctor.rules).expect("rules");
    if let Some(protection) = doctor.protection {
        fs::write(tools.path().join("protection.json"), protection).expect("protection");
    }
    let search_path = if doctor.has_gh {
        format!("{}:{SYSTEM_PATH}", tools.path().display())
    } else {
        SYSTEM_PATH.to_string()
    };
    let output = Command::new(env!("CARGO_BIN_EXE_vigilante"))
        .arg("--repository")
        .arg(repository.directory.path())
        .arg("doctor")
        .env("PATH", search_path)
        .output()
        .expect("vigilante runs");
    Run {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn unchecked(stdout: &str) -> Vec<&str> {
    stdout
        .lines()
        .filter(|line| line.starts_with("- [ ]"))
        .collect()
}

#[test]
fn doctor_passes_when_code_owners_and_branch_rules_protect_the_lock() {
    let run = run_doctor(Doctor {
        codeowners: ALL_OWNED,
        rules: PROTECTED_RULES,
        protection: None,
        has_gh: true,
    });

    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert_eq!(unchecked(&run.stdout), Vec::<&str>::new(), "{}", run.stdout);
    assert!(
        run.stdout
            .contains("- [x] `vigilante.lock` has the code owners `@acme/security`."),
        "{}",
        run.stdout
    );
}

#[test]
fn doctor_lists_each_missing_protection() {
    let run = run_doctor(Doctor {
        codeowners: "* @acme/everyone\nvigilante.lock\n/.github/ @acme/security\n",
        rules: r#"[{"type":"pull_request","parameters":{"require_code_owner_review":false}},{"type":"required_status_checks","parameters":{"required_status_checks":[{"context":"test"}]}}]"#,
        protection: None,
        has_gh: true,
    });

    assert_eq!(run.code, 1, "{}{}", run.stdout, run.stderr);
    assert_eq!(
        unchecked(&run.stdout),
        vec![
            "- [ ] `vigilante.lock` has no code owner in `.github/CODEOWNERS`.",
            "- [ ] `main` does not require a review from code owners.",
            "- [ ] `main` does not require a check with `vigilante` in its name. The required checks are `test`.",
        ],
        "{}",
        run.stdout
    );
}

#[test]
fn doctor_reads_classic_branch_protection() {
    let run = run_doctor(Doctor {
        codeowners: ALL_OWNED,
        rules: "[]",
        protection: Some(
            r#"{"required_pull_request_reviews":{"require_code_owner_reviews":true},"required_status_checks":{"contexts":["Vigilante / vigilante"]}}"#,
        ),
        has_gh: true,
    });

    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert_eq!(unchecked(&run.stdout), Vec::<&str>::new(), "{}", run.stdout);
}

#[test]
fn doctor_fails_when_it_cannot_read_the_branch_rules() {
    let run = run_doctor(Doctor {
        codeowners: ALL_OWNED,
        rules: PROTECTED_RULES,
        protection: None,
        has_gh: false,
    });

    assert_eq!(run.code, 1, "{}{}", run.stdout, run.stderr);
    assert_eq!(
        unchecked(&run.stdout),
        vec!["- [ ] Vigilante cannot read the branch rules: `gh` is not installed."],
        "{}",
        run.stdout
    );
}
