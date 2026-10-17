use crate::support::{TestRepository, finding_lines};

const NAME_BASED: &str =
    "[engines]\njavascript = \"deep\"\npython = \"deep\"\nrust = \"deep\"\njava = \"deep\"\n";
const CARGO_MANIFEST: &str = "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
const CARGO_LOCK: &str = "version = 4\n\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\n";

fn findings_of(files: &[(&str, &str)]) -> Vec<String> {
    let repository = TestRepository::new();
    repository.write(&[("vigilante.toml", NAME_BASED)]);
    repository.write(files);
    repository.commit("code");
    finding_lines(&repository.check())
}

#[test]
fn a_fetch_through_global_this_is_network() {
    assert_eq!(
        findings_of(&[(
            "app.js",
            "export function send(url) { return globalThis.fetch(url); }\n"
        )]),
        vec!["code . network locked=false denied=false"]
    );
}

#[test]
fn a_fetch_through_a_constant_alias_is_network() {
    assert_eq!(
        findings_of(&[(
            "app.js",
            "const request = fetch;\nexport function send(url) { return request(url); }\n"
        )]),
        vec!["code . network locked=false denied=false"]
    );
}

#[test]
fn a_local_function_named_fetch_is_not_network() {
    assert_eq!(
        findings_of(&[(
            "app.js",
            "function fetch(value) { return value; }\nexport function read(value) { return fetch(value); }\n"
        )]),
        Vec::<String>::new()
    );
}

#[test]
fn an_import_of_a_constant_module_name_is_followed() {
    assert_eq!(
        findings_of(&[(
            "app.js",
            "const moduleName = \"node:child_process\";\nexport async function run(command) { const child = await import(moduleName); return child.exec(command); }\n"
        )]),
        vec!["code . exec locked=false denied=false"]
    );
}

#[test]
fn an_imported_python_system_function_is_exec() {
    assert_eq!(
        findings_of(&[(
            "app.py",
            "from os import system\n\n\ndef run(command):\n    return system(command)\n"
        )]),
        vec!["code . exec locked=false denied=false"]
    );
}

#[test]
fn an_asyncio_subprocess_is_exec() {
    assert_eq!(
        findings_of(&[(
            "app.py",
            "import asyncio\n\n\nasync def run(command):\n    return await asyncio.create_subprocess_exec(command)\n"
        )]),
        vec!["code . exec locked=false denied=false"]
    );
}

#[test]
fn a_subprocess_helper_that_only_formats_text_is_not_exec() {
    assert_eq!(
        findings_of(&[(
            "app.py",
            "from subprocess import list2cmdline\n\n\ndef quote(arguments):\n    return list2cmdline(arguments)\n"
        )]),
        Vec::<String>::new()
    );
}

#[test]
fn a_java_runtime_in_a_local_variable_is_exec() {
    assert_eq!(
        findings_of(&[(
            "App.java",
            "class App { Process run(String command) throws Exception { Runtime runtime = Runtime.getRuntime(); return runtime.exec(command); } }\n"
        )]),
        vec!["code . exec locked=false denied=false"]
    );
}

#[test]
fn a_fully_qualified_java_socket_is_network() {
    assert_eq!(
        findings_of(&[(
            "App.java",
            "class App { Object connect(String host) throws Exception { return new java.net.Socket(host, 443); } }\n"
        )]),
        vec!["code . network locked=false denied=false"]
    );
}

#[test]
fn a_rust_command_through_a_module_alias_is_exec() {
    assert_eq!(
        findings_of(&[
            ("Cargo.toml", CARGO_MANIFEST),
            ("Cargo.lock", CARGO_LOCK),
            (
                "src/lib.rs",
                "use std::process as processes;\npub fn run() { processes::Command::new(\"id\").status().unwrap(); }\n"
            ),
        ]),
        vec!["code . exec locked=false denied=false"]
    );
}

#[test]
fn a_subprocess_function_that_runs_a_program_is_still_exec() {
    assert_eq!(
        findings_of(&[(
            "app.py",
            "from subprocess import run\n\n\ndef build(command):\n    return run(command)\n"
        )]),
        vec!["code . exec locked=false denied=false"]
    );
}
