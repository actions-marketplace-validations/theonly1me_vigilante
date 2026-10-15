use serde_json::Value;

use crate::npm_registry::report_of;
use crate::python_fixtures::summaries;
use crate::rust_fixtures::{Change, FakeCrate, RUNNER, Registry, SPAWNER, app_manifest, diff};

const RUNNING_MAIN: &str = "fn main() {\n    runner::start();\n}\n";
const NET: FakeCrate = FakeCrate {
    name: "net-lib",
    version: "0.3.0",
    dependencies: &[],
    files: &[
        (
            "src/lib.rs",
            "pub mod client;\n\npub use self::client::Client;\n",
        ),
        (
            "src/client.rs",
            "use std::net::TcpStream;\n\npub struct Client;\n\nimpl Client {\n    pub fn new() -> Self {\n        Client\n    }\n\n    pub fn send(&self) {\n        TcpStream::connect(\"example.com:80\");\n    }\n}\n",
        ),
    ],
};

#[test]
fn a_new_crate_that_runs_programs_is_a_note_with_its_call_path() {
    let registry = Registry::serve(&[&RUNNER, &SPAWNER]);

    let run = diff(Change {
        base: &[
            ("Cargo.toml", &app_manifest(&["runner"])),
            ("Cargo.lock", &registry.lockfile(&[&RUNNER, &SPAWNER])),
            ("vigilante.toml", &registry.allow_config()),
        ],
        head: &[("src/main.rs", RUNNING_MAIN)],
    });

    let report = report_of(&run);
    let path =
        "src/main.main > runner/src/work.start > spawner/src/lib.spawn_it > std::process.Command";
    assert_eq!(
        report["findings"],
        Value::Array(Vec::new()),
        "{}",
        run.stderr
    );
    assert_eq!(
        summaries(&report["notes"]),
        vec![
            format!(". exec runner src/main.rs:2 {path}"),
            format!(". exec runner src/main.rs:2 {path}"),
            format!(". exec spawner src/main.rs:2 {path}"),
        ],
        "{}",
        run.stderr
    );
}

#[test]
fn a_type_that_a_crate_reexports_leads_to_its_methods() {
    let registry = Registry::serve(&[&NET]);

    let run = diff(Change {
        base: &[
            ("Cargo.toml", &app_manifest(&["net-lib"])),
            ("Cargo.lock", &registry.lockfile(&[&NET])),
            ("vigilante.toml", &registry.allow_config()),
        ],
        head: &[(
            "src/main.rs",
            "use net_lib::Client;\n\nfn main() {\n    let client = Client::new();\n    client.send();\n}\n",
        )],
    });

    let notes = summaries(&report_of(&run)["notes"]);
    assert!(
        notes.contains(&". network net-lib src/main.rs:5 src/main.main > net-lib/src/client.Client.send > std::net.TcpStream".to_string()),
        "{notes:#?}\n{}", run.stderr
    );
}

#[test]
fn a_crate_that_does_not_match_its_lockfile_checksum_stops_the_review() {
    let registry = Registry::serve(&[&RUNNER, &SPAWNER]);
    let lockfile = registry.lockfile(&[&RUNNER, &SPAWNER]);
    let checksums: Vec<&str> = lockfile
        .lines()
        .filter_map(|line| line.strip_prefix("checksum = "))
        .collect();
    let tampered = lockfile.replacen(checksums[0], checksums[1], 1);

    let run = diff(Change {
        base: &[
            ("Cargo.toml", &app_manifest(&["runner"])),
            ("Cargo.lock", &tampered),
            ("vigilante.toml", &registry.allow_config()),
        ],
        head: &[("src/main.rs", RUNNING_MAIN)],
    });

    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(
        run.stderr.contains("does not match the checksum"),
        "{}",
        run.stderr
    );
}

#[test]
fn a_type_from_a_grouped_use_of_a_privileged_module_is_a_sink() {
    let launcher = FakeCrate {
        name: "launcher",
        version: "1.0.0",
        dependencies: &[],
        files: &[(
            "src/lib.rs",
            "use std::process::{self, Command};\n\npub fn launch() {\n    Command::new(\"ls\");\n}\n",
        )],
    };
    let registry = Registry::serve(&[&launcher]);

    let run = diff(Change {
        base: &[
            ("Cargo.toml", &app_manifest(&["launcher"])),
            ("Cargo.lock", &registry.lockfile(&[&launcher])),
            ("vigilante.toml", &registry.allow_config()),
        ],
        head: &[("src/main.rs", "fn main() {\n    launcher::launch();\n}\n")],
    });

    let note = ". exec launcher src/main.rs:2 src/main.main > launcher/src/lib.launch > std::process.Command".to_string();
    assert_eq!(
        summaries(&report_of(&run)["notes"]),
        vec![note.clone(), note],
        "{}",
        run.stderr
    );
}

#[test]
fn a_method_of_a_type_is_found_by_the_type_and_not_by_the_name_alone() {
    let maker = FakeCrate {
        name: "maker",
        version: "1.0.0",
        dependencies: &[],
        files: &[
            (
                "src/lib.rs",
                "mod send;\n\npub struct Builder;\npub struct Client;\npub struct Shell;\n\nimpl Shell {\n    pub fn send() {\n        std::process::Command::new(\"ls\");\n    }\n}\n\npub fn ping() {\n    Client::send();\n}\n",
            ),
            (
                "src/send.rs",
                "impl super::Client {\n    pub fn send() {\n        std::net::TcpStream::connect(\"example.com:80\");\n    }\n}\n",
            ),
        ],
    };
    let registry = Registry::serve(&[&maker]);

    let run = diff(Change {
        base: &[
            ("Cargo.toml", &app_manifest(&["maker"])),
            ("Cargo.lock", &registry.lockfile(&[&maker])),
            ("vigilante.toml", &registry.allow_config()),
        ],
        head: &[(
            "src/main.rs",
            "fn main() {\n    maker::Builder::send();\n    maker::ping();\n}\n",
        )],
    });

    let note = ". network maker src/main.rs:3 src/main.main > maker/src/lib.ping > maker/src/send.Client.send > std::net::TcpStream::connect".to_string();
    assert_eq!(
        summaries(&report_of(&run)["notes"]),
        vec![note.clone(), note],
        "{}",
        run.stderr
    );
}
