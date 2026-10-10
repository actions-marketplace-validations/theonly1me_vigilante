use vigilante::Lock;

use crate::support::{TestRepository, finding_lines};

#[test]
fn check_fails_on_unlocked_powers_and_on_stale_lock_entries() {
    let repository = TestRepository::new();
    repository.write(&[
        ("services/run.py", "import subprocess\n"),
        (
            Lock::FILE,
            "version = 1\n\n[code.services]\npowers = [\"network\"]\n",
        ),
    ]);

    let review = repository.check();

    assert_eq!(
        finding_lines(&review),
        ["code services exec locked=false denied=false"]
    );
    let stale: Vec<String> = review
        .stale
        .iter()
        .map(|entry| format!("{} {}", entry.scope.path, entry.power))
        .collect();
    assert_eq!(stale, ["services network"]);
    assert!(!review.passed());
}

#[test]
fn a_deny_rule_fails_even_in_report_mode_and_with_a_lock_entry() {
    let repository = TestRepository::new();
    repository.write(&[
        ("services/run.py", "import subprocess\n"),
        (
            "vigilante.toml",
            "[policy]\nmode = \"report\"\ndeny = [{ scope = \"services\", capability = \"exec\" }]\n",
        ),
        (
            Lock::FILE,
            "version = 1\n\n[code.services]\npowers = [\"exec\"]\n",
        ),
    ]);

    let review = repository.check();

    assert_eq!(
        finding_lines(&review),
        ["code services exec locked=true denied=true"]
    );
    assert!(!review.passed());
}

#[test]
fn report_mode_passes_with_unlocked_powers() {
    let repository = TestRepository::new();
    repository.write(&[
        ("services/run.py", "import subprocess\n"),
        ("vigilante.toml", "[policy]\nmode = \"report\"\n"),
    ]);

    let review = repository.check();

    assert_eq!(
        finding_lines(&review),
        ["code services exec locked=false denied=false"]
    );
    assert!(review.passed());
}
