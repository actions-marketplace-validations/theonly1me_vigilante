use crate::npm_registry::report_of;
use crate::python_fixtures::summaries;
use crate::rust_fixtures::{Change, app_manifest, diff};

const LOCKFILE: &str = "version = 4\n\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\n";

fn findings_after(head: &[(&str, &str)]) -> Vec<String> {
    let manifest = app_manifest(&[]);
    let run = diff(Change {
        base: &[("Cargo.toml", &manifest), ("Cargo.lock", LOCKFILE)],
        head,
    });
    summaries(&report_of(&run)["findings"])
}

#[test]
fn own_code_that_reaches_exec_through_another_crate_and_module_paths_is_a_finding() {
    let run = diff(Change {
        base: &[
            (
                "Cargo.toml",
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\n\n[dependencies]\ntools = { path = \"tools\" }\n",
            ),
            (
                "Cargo.lock",
                "version = 4\n\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\ndependencies = [\n \"tools\",\n]\n\n[[package]]\nname = \"tools\"\nversion = \"0.1.0\"\n",
            ),
            (
                "tools/Cargo.toml",
                "[package]\nname = \"tools\"\nversion = \"0.1.0\"\n",
            ),
            ("tools/src/lib.rs", "pub mod shell;\nmod quote;\n"),
            (
                "tools/src/shell.rs",
                "pub fn run() {\n    super::quote::spawn();\n}\n",
            ),
            (
                "tools/src/quote.rs",
                "use std::process::Command;\n\npub fn spawn() {\n    Command::new(\"make\");\n}\n",
            ),
            ("src/lib.rs", "pub fn build() {}\n"),
        ],
        head: &[(
            "src/lib.rs",
            "pub fn build() {\n    tools::shell::run();\n}\n",
        )],
    });

    assert_eq!(
        summaries(&report_of(&run)["findings"]),
        vec![
            ". exec own src/lib.rs:2 src/lib.build > tools/src/shell.run > tools/src/quote.spawn > std::process.Command"
        ],
        "{}",
        run.stderr
    );
}

#[test]
fn unsafe_code_in_tests_is_not_a_finding() {
    let findings = findings_after(&[(
        "src/lib.rs",
        "pub fn add(left: u8, right: u8) -> u8 {\n    left + right\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn raw() {\n        unsafe { std::ptr::null::<u8>().read() };\n    }\n}\n",
    )]);

    assert_eq!(findings, Vec::<String>::new());
}

#[test]
fn module_files_that_only_tests_declare_are_not_findings() {
    let findings = findings_after(&[
        (
            "src/lib.rs",
            "pub fn add() {}\n\n#[cfg(test)]\nmod checks;\nmod shell;\n",
        ),
        (
            "src/shell.rs",
            "pub fn spawn() {\n    std::process::Command::new(\"make\");\n}\n",
        ),
        (
            "src/checks.rs",
            "mod support;\n\npub fn connect() {\n    std::net::TcpStream::connect(\"example.com:80\");\n}\n",
        ),
        (
            "src/checks/support.rs",
            "pub fn first(values: &[u8]) -> u8 {\n    unsafe { *values.as_ptr() }\n}\n",
        ),
    ]);

    assert_eq!(
        findings,
        vec![". exec own src/shell.rs:2 src/shell.spawn > std::process::Command"]
    );
}

#[test]
fn an_import_that_is_never_used_is_not_a_power() {
    let findings = findings_after(&[("src/lib.rs", "use reqwest::Client;\n\npub fn build() {}\n")]);

    assert_eq!(findings, Vec::<String>::new());
}

#[test]
fn only_the_names_that_a_rule_matches_in_a_use_tree_are_sinks() {
    let findings = findings_after(&[(
        "src/lib.rs",
        "use std::{net::TcpStream, process::Command, time::Duration};\n\npub fn wait() {\n    std::thread::sleep(Duration::from_secs(1));\n}\n\npub fn run() {\n    Command::new(\"ls\");\n}\n\npub fn connect() {\n    TcpStream::connect(\"example.com:80\");\n}\n",
    )]);

    assert_eq!(
        findings,
        vec![
            ". exec own src/lib.rs:8 src/lib.run > std::process.Command",
            ". network own src/lib.rs:12 src/lib.connect > std::net.TcpStream"
        ]
    );
}

#[test]
fn unsafe_code_outside_tests_is_a_native_finding() {
    let findings = findings_after(&[(
        "src/lib.rs",
        "pub fn first(values: &[u8]) -> u8 {\n    unsafe { *values.as_ptr() }\n}\n",
    )]);

    assert_eq!(
        findings,
        vec![". native own src/lib.rs:2 src/lib.first > unsafe { *values.as_ptr"]
    );
}

#[test]
fn crates_that_a_workspace_renames_are_found_by_their_library_names() {
    let run = diff(Change {
        base: &[
            (
                "Cargo.toml",
                "[workspace]\nmembers = [\"app\", \"tool\", \"helper\"]\n\n[workspace.dependencies]\ntool = { path = \"tool\", package = \"tool-rs\" }\n",
            ),
            (
                "Cargo.lock",
                "version = 4\n\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\ndependencies = [\n \"helper-rs\",\n \"tool-rs\",\n]\n\n[[package]]\nname = \"helper-rs\"\nversion = \"0.1.0\"\n\n[[package]]\nname = \"tool-rs\"\nversion = \"0.1.0\"\n",
            ),
            (
                "app/Cargo.toml",
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\n\n[dependencies]\ntool = { workspace = true }\nhelper-rs = { path = \"../helper\" }\n",
            ),
            ("app/src/lib.rs", "pub fn build() {}\n"),
            (
                "tool/Cargo.toml",
                "[package]\nname = \"tool-rs\"\nversion = \"0.1.0\"\n\n[lib]\nname = \"tool\"\n",
            ),
            (
                "tool/src/lib.rs",
                "pub fn run() {\n    std::process::Command::new(\"make\");\n}\n",
            ),
            (
                "helper/Cargo.toml",
                "[package]\nname = \"helper-rs\"\nversion = \"0.1.0\"\n\n[lib]\nname = \"helper\"\n",
            ),
            (
                "helper/src/lib.rs",
                "pub fn fetch() {\n    std::net::TcpStream::connect(\"example.com:80\");\n}\n",
            ),
        ],
        head: &[(
            "app/src/lib.rs",
            "pub fn build() {\n    tool::run();\n    helper::fetch();\n}\n",
        )],
    });

    assert_eq!(
        summaries(&report_of(&run)["findings"]),
        vec![
            "app exec own app/src/lib.rs:2 app/src/lib.build > tool/src/lib.run > std::process::Command",
            "app network own app/src/lib.rs:3 app/src/lib.build > helper/src/lib.fetch > std::net::TcpStream::connect",
        ],
        "{}",
        run.stderr
    );
}
