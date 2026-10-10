use std::collections::BTreeSet;

use vigilante::Lock;

use crate::support::{TestRepository, expected, finding_lines};

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

const POWERS_FROM_CAPSLOCK: &[&str] = &[
    "arbitrary_execution",
    "files",
    "modify_system_state",
    "operating_system",
    "read_system_state",
    "runtime",
    "system_calls",
    "unanalyzed",
];

#[test]
fn a_lock_can_list_every_power_that_capslock_reports() {
    let repository = TestRepository::new();
    let quoted: Vec<String> = POWERS_FROM_CAPSLOCK
        .iter()
        .map(|power| format!("\"{power}\""))
        .collect();
    let lock = format!(
        "version = 1\n\n[code.services]\npowers = [{}]\n",
        quoted.join(", ")
    );
    repository.write(&[("services/README.md", "# services\n"), (Lock::FILE, &lock)]);

    let review = repository.check();

    let stale: BTreeSet<String> = review
        .stale
        .iter()
        .map(|entry| entry.power.to_string())
        .collect();
    assert_eq!(stale, expected(POWERS_FROM_CAPSLOCK));
}
