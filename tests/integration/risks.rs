use vigilante::Review;

use crate::support::{Branches, TestRepository};

const NAME_BASED: &str = "[engines]\npython = \"deep\"\njavascript = \"deep\"\n";
pub const FIXED_DATE: &str =
    "import subprocess\n\n\ndef export(request):\n    return subprocess.run([\"date\"])\n";
const SHELL_WITH_INPUT: &str = "import subprocess\n\n\ndef export(request):\n    return subprocess.run(f\"date {request.args['day']}\", shell=True)\n";

pub struct Change<'a> {
    pub base: &'a [(&'a str, &'a str)],
    pub head: &'a [(&'a str, &'a str)],
}

pub fn review_of(change: Change) -> Review {
    let repository = TestRepository::new();
    repository.write(&[("vigilante.toml", NAME_BASED)]);
    repository.write(change.base);
    let base = repository.commit("base");
    repository.write(change.head);
    let head = repository.commit("change");
    repository.diff(Branches {
        base: &base,
        head: &head,
    })
}

pub fn risk_lines(review: &Review) -> Vec<String> {
    review
        .risks
        .iter()
        .map(|risk| {
            format!(
                "{} {} blocks={} approved={}",
                risk.scope.path, risk.identity, risk.blocks, risk.approved
            )
        })
        .collect()
}

#[test]
fn a_shell_call_with_request_input_blocks_in_a_package_that_already_runs_programs() {
    let review = review_of(Change {
        base: &[("app.py", FIXED_DATE)],
        head: &[("app.py", SHELL_WITH_INPUT)],
    });

    assert_eq!(
        risk_lines(&review),
        vec![". exec:app.export:subprocess.run:untrusted blocks=true approved=false"]
    );
    assert!(review.findings.is_empty());
    assert!(!review.passed());
}

#[test]
fn a_call_that_becomes_dynamic_is_shown_and_does_not_block() {
    let review = review_of(Change {
        base: &[("app.py", FIXED_DATE)],
        head: &[(
            "app.py",
            "import subprocess\n\n\ndef export(day):\n    return subprocess.run([\"date\", day])\n",
        )],
    });

    assert_eq!(
        risk_lines(&review),
        vec![". exec:app.export:subprocess.run:dynamic blocks=false approved=false"]
    );
    assert!(review.passed());
}

#[test]
fn a_risk_that_the_lock_approves_passes() {
    let review = review_of(Change {
        base: &[("app.py", FIXED_DATE)],
        head: &[
            ("app.py", SHELL_WITH_INPUT),
            (
                "vigilante.lock",
                "version = 1\n\n[code.\".\"]\npowers = [\"exec\"]\nrisks = [\"exec:app.export:subprocess.run:untrusted\"]\n",
            ),
        ],
    });

    assert_eq!(
        risk_lines(&review),
        vec![". exec:app.export:subprocess.run:untrusted blocks=true approved=true"]
    );
    assert!(review.passed());
}

#[test]
fn eval_of_request_input_blocks() {
    let review = review_of(Change {
        base: &[("app.py", "def execute(request):\n    return 0\n")],
        head: &[(
            "app.py",
            "def execute(request):\n    return eval(request.args[\"expression\"])\n",
        )],
    });

    assert_eq!(
        risk_lines(&review),
        vec![". dynamic_code:app.execute:eval:untrusted blocks=true approved=false"]
    );
    assert!(!review.passed());
}

#[test]
fn request_input_that_reaches_a_command_through_another_function_blocks() {
    let review = review_of(Change {
        base: &[("app.py", FIXED_DATE)],
        head: &[(
            "app.py",
            "import subprocess\n\n\ndef run(command):\n    return subprocess.run(command, shell=True)\n\n\ndef export(request):\n    return run(request.args[\"command\"])\n",
        )],
    });

    assert_eq!(
        risk_lines(&review),
        vec![". exec:app.run:subprocess.run:untrusted blocks=true approved=false"]
    );
}

#[test]
fn a_javascript_exec_with_query_input_blocks() {
    let review = review_with_javascript(
        "const { exec } = require(\"child_process\");\nexports.list = (req, res) => exec(\"ls \" + req.query.folder);\n",
    );

    assert_eq!(
        risk_lines(&review),
        vec![". exec:index.list:child_process.exec:untrusted blocks=true approved=false"]
    );
}

#[test]
fn equal_risks_in_one_function_are_one_risk() {
    let review = review_of(Change {
        base: &[("app.py", FIXED_DATE)],
        head: &[(
            "app.py",
            "import subprocess\n\n\ndef export(day):\n    subprocess.run([\"date\", day])\n    return subprocess.run([\"date\", day])\n",
        )],
    });

    assert_eq!(risk_lines(&review).len(), 1);
}

pub fn review_with_javascript(head: &str) -> Review {
    review_of(Change {
        base: &[
            ("package.json", "{\"name\": \"app\"}\n"),
            ("index.js", "exports.list = () => 0;\n"),
        ],
        head: &[("index.js", head)],
    })
}
