use crate::risks::{Change, review_of, risk_lines};

const NODE_BASE: &str = "const { exec, execSync } = require(\"child_process\");\nexports.build = (folder) => execSync(\"npm run build\");\n";
const PYTHON_BASE: &str =
    "import subprocess\n\n\ndef build(folder, command):\n    return subprocess.run([\"make\"])\n";

fn javascript_lines(head: &str) -> Vec<String> {
    risk_lines(&review_of(Change {
        base: &[
            ("package.json", "{\"name\": \"app\"}\n"),
            ("index.js", NODE_BASE),
        ],
        head: &[("index.js", head)],
    }))
}

fn python_lines(call: &str) -> Vec<String> {
    let head = format!("import subprocess\n\n\ndef build(folder, command):\n    return {call}\n");
    risk_lines(&review_of(Change {
        base: &[("app.py", PYTHON_BASE)],
        head: &[("app.py", &head)],
    }))
}

#[test]
fn an_options_object_does_not_make_a_constant_command_change() {
    let head = "const { exec, execSync } = require(\"child_process\");\nexports.build = (folder) => execSync(\"npm run build\", { cwd: folder, stdio: \"inherit\" });\n";

    assert_eq!(javascript_lines(head), Vec::<String>::new());
}

#[test]
fn a_callback_does_not_make_a_constant_command_change() {
    let head = "const { exec, execSync } = require(\"child_process\");\nexports.build = (folder) => exec(\"git status\", (error, output) => output);\n";

    assert_eq!(javascript_lines(head), Vec::<String>::new());
}

#[test]
fn a_working_folder_does_not_make_a_constant_shell_command_change() {
    assert_eq!(
        python_lines("subprocess.run(\"make\", shell=True, cwd=folder)"),
        Vec::<String>::new()
    );
}

#[test]
fn a_command_in_the_args_keyword_is_still_a_shell_risk() {
    assert_eq!(
        python_lines("subprocess.run(args=command, shell=True, cwd=folder)"),
        vec![". exec:app.build:subprocess.run:shell blocks=true approved=false"]
    );
}
