use crate::doctor::{ALL_OWNED, Doctor, PROTECTED_RULES, run_doctor, unchecked};

#[test]
fn doctor_checks_the_rule_folders_that_vigilante_toml_configures() {
    let run = run_doctor(Doctor {
        codeowners: ALL_OWNED,
        files: &[
            ("vigilante.toml", "[rules]\npaths = [\"security/rules\"]\n"),
            ("security/rules/client.yml", "id: client\n"),
        ],
        rules: PROTECTED_RULES,
        protection: None,
        has_gh: true,
    });

    assert_eq!(run.code, 1, "{}{}", run.stdout, run.stderr);
    assert_eq!(
        unchecked(&run.stdout),
        vec!["- [ ] `security/rules/` has no code owner in `.github/CODEOWNERS`."],
        "{}",
        run.stdout
    );
}

#[test]
fn doctor_checks_each_existing_file_in_a_protected_folder() {
    let run = run_doctor(Doctor {
        codeowners: "vigilante.lock @acme/security\nvigilante.toml @acme/security\n/.vigilante/ @acme/security\n/.github/CODEOWNERS @acme/security\n/.github/workflows/vigilante.yml @acme/security\n",
        files: &[
            (".github/workflows/vigilante.yml", "name: Vigilante\n"),
            (".github/workflows/ci.yml", "name: CI\n"),
        ],
        rules: PROTECTED_RULES,
        protection: None,
        has_gh: true,
    });

    assert_eq!(run.code, 1, "{}{}", run.stdout, run.stderr);
    assert_eq!(
        unchecked(&run.stdout),
        vec![
            "- [ ] `.github/workflows/` has no code owner for `.github/workflows/ci.yml` in `.github/CODEOWNERS`."
        ],
        "{}",
        run.stdout
    );
}

#[test]
fn doctor_fails_when_vigilante_toml_is_invalid() {
    let run = run_doctor(Doctor {
        codeowners: ALL_OWNED,
        files: &[(
            "vigilante.toml",
            "[rules]\nfolders = [\"security/rules\"]\n",
        )],
        rules: PROTECTED_RULES,
        protection: None,
        has_gh: true,
    });

    assert_eq!(run.code, 1, "{}{}", run.stdout, run.stderr);
    assert!(
        unchecked(&run.stdout).iter().any(|line| line
            .starts_with("- [ ] Vigilante cannot read the rule folders: invalid vigilante.toml")),
        "{}",
        run.stdout
    );
}
