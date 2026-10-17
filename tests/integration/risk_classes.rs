use crate::risks::{Change, FIXED_DATE, review_of, review_with_javascript, risk_lines};

#[test]
fn a_shell_command_built_from_a_parameter_blocks() {
    let review = review_of(Change {
        base: &[("app.py", FIXED_DATE)],
        head: &[(
            "app.py",
            "import subprocess\n\n\ndef export(day):\n    return subprocess.run(\"date \" + day, shell=True)\n",
        )],
    });

    assert_eq!(
        risk_lines(&review),
        vec![". exec:app.export:subprocess.run:shell blocks=true approved=false"]
    );
    assert!(!review.passed());
}

#[test]
fn a_risk_that_only_moves_to_another_line_is_not_new() {
    let shell = "def export(request):\n    return subprocess.run(f\"date {request.args['day']}\", shell=True)\n";
    let review = review_of(Change {
        base: &[("app.py", &format!("import subprocess\n\n\n{shell}"))],
        head: &[(
            "app.py",
            &format!("import subprocess\n\nVERSION = 2\n\n\n{shell}"),
        )],
    });

    assert_eq!(risk_lines(&review), Vec::<String>::new());
}

#[test]
fn a_constant_command_in_a_new_function_is_not_a_changed_risk() {
    let review = review_of(Change {
        base: &[("app.py", FIXED_DATE)],
        head: &[(
            "app.py",
            &format!("{FIXED_DATE}\n\ndef uptime():\n    return subprocess.run([\"uptime\"])\n"),
        )],
    });

    assert_eq!(risk_lines(&review), Vec::<String>::new());
    assert!(review.passed());
}

#[test]
fn a_javascript_exec_that_builds_a_command_from_a_parameter_blocks() {
    let review = review_with_javascript(
        "const { exec } = require(\"child_process\");\nexports.list = (folder) => exec(\"ls \" + folder);\n",
    );

    assert_eq!(
        risk_lines(&review),
        vec![". exec:index.list:child_process.exec:shell blocks=true approved=false"]
    );
}

#[test]
fn an_explicit_shell_with_a_parameter_command_blocks() {
    let review = review_of(Change {
        base: &[("app.py", FIXED_DATE)],
        head: &[(
            "app.py",
            "import subprocess\n\n\ndef export(command):\n    return subprocess.run([\"sh\", \"-c\", command])\n",
        )],
    });

    assert_eq!(
        risk_lines(&review),
        vec![". exec:app.export:subprocess.run:shell blocks=true approved=false"]
    );
}

#[test]
fn a_name_that_only_starts_like_request_input_is_not_untrusted() {
    let review = review_of(Change {
        base: &[("app.py", FIXED_DATE)],
        head: &[(
            "app.py",
            "import subprocess\n\n\ndef export(request):\n    return subprocess.run([\"date\", request.database])\n",
        )],
    });

    assert_eq!(
        risk_lines(&review),
        vec![". exec:app.export:subprocess.run:dynamic blocks=false approved=false"]
    );
}

#[test]
fn command_line_arguments_are_local_input_and_do_not_block() {
    let review = review_with_javascript(
        "const { spawnSync } = require(\"child_process\");\nexports.list = () => spawnSync(\"ls\", process.argv.slice(2));\n",
    );

    assert_eq!(
        risk_lines(&review),
        vec![". exec:index.list:child_process.spawnSync:dynamic blocks=false approved=false"]
    );
}
