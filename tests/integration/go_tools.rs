use std::collections::BTreeSet;

use crate::go_fixtures::{APP_MODULE, APP_MODULE_FILE, Call, FETCH_SOURCE, Reach, capslock_output};
use crate::go_support::{FakeTools, GoRun, Tools, report_of, run_vigilante};
use crate::support::{TestRepository, expected};

const PAGE: Call = Call {
    function: "example.com/app/fetch.Page",
    package: "example.com/app/fetch",
    site: None,
};
const PRIVILEGED_CALL: Call = Call {
    function: "os.Something",
    package: "os",
    site: Some(("fetch.go", 3)),
};
const CAPSLOCK_TO_VIGILANTE: &[(&str, &str)] = &[
    ("ARBITRARY_EXECUTION", "arbitrary_execution"),
    ("CGO", "native:cgo"),
    ("EXEC", "exec"),
    ("FILES", "files"),
    ("MODIFY_SYSTEM_STATE/ENV", "modify_system_state:env"),
    ("NETWORK", "network"),
    ("OPERATING_SYSTEM", "operating_system"),
    ("READ_SYSTEM_STATE", "read_system_state"),
    ("REFLECT", "dynamic_code:reflect"),
    ("RUNTIME", "runtime"),
    ("SYSTEM_CALLS", "system_calls"),
    ("UNANALYZED", "unanalyzed"),
    ("UNSAFE_POINTER", "native:unsafe_pointer"),
];

fn repository_with(capabilities: &[&'static str]) -> TestRepository {
    let reaches: Vec<Reach> = capabilities
        .iter()
        .map(|capability| Reach {
            package: "example.com/app/fetch",
            capability,
            calls: &[PAGE, PRIVILEGED_CALL],
        })
        .collect();
    let repository = TestRepository::new();
    repository.write(&[
        (APP_MODULE, APP_MODULE_FILE),
        ("app/fetch/fetch.go", FETCH_SOURCE),
        ("app/capslock.json", &capslock_output(&reaches)),
    ]);
    repository
}

fn scan(
    repository: &TestRepository,
    available: Tools,
    environment: &[(&str, &str)],
) -> crate::support::Run {
    let tools = FakeTools::new();
    run_vigilante(GoRun {
        repository,
        tools: &tools,
        available,
        arguments: &["scan", "--format", "json"],
        environment,
    })
}

#[test]
fn each_capslock_capability_maps_to_a_vigilante_power() {
    let names: Vec<&str> = CAPSLOCK_TO_VIGILANTE
        .iter()
        .map(|(name, _)| *name)
        .collect();
    let repository = repository_with(&names);

    let run = scan(&repository, Tools::Fake, &[]);

    assert_eq!(run.code, 0, "{}", run.stderr);
    let report = report_of(&run);
    let powers: BTreeSet<String> = report
        .as_array()
        .expect("scopes")
        .iter()
        .filter(|scope| scope["scope"] == "app/fetch")
        .flat_map(|scope| scope["powers"].as_array().cloned().unwrap_or_default())
        .map(|power| power["power"].as_str().unwrap_or_default().to_string())
        .collect();
    let wanted: Vec<&str> = CAPSLOCK_TO_VIGILANTE
        .iter()
        .map(|(_, power)| *power)
        .collect();
    assert_eq!(powers, expected(&wanted));
}

#[test]
fn an_unknown_capslock_capability_stops_the_scan() {
    let repository = repository_with(&["TELEPORT"]);

    let run = scan(&repository, Tools::Fake, &[]);

    assert_eq!(run.code, 2);
    assert!(run.stderr.contains("TELEPORT"), "{}", run.stderr);
}

#[test]
fn a_missing_capslock_stops_the_scan_with_the_install_command() {
    let repository = repository_with(&["NETWORK"]);

    let run = scan(&repository, Tools::Missing, &[]);

    assert_eq!(run.code, 2);
    assert!(
        run.stderr
            .contains("go install github.com/google/capslock/cmd/capslock@v0.3.3"),
        "{}",
        run.stderr
    );
}

#[test]
fn an_old_capslock_stops_the_scan() {
    let repository = repository_with(&["NETWORK"]);

    let run = scan(
        &repository,
        Tools::Fake,
        &[("FAKE_CAPSLOCK_VERSION", "v0.2.8")],
    );

    assert_eq!(run.code, 2);
    assert!(run.stderr.contains("v0.2.8"), "{}", run.stderr);
}

#[test]
fn the_patterns_engine_reads_go_without_capslock() {
    let repository = repository_with(&["NETWORK"]);
    repository.write(&[
        ("vigilante.toml", "[engines]\ngo = \"patterns\"\n"),
        (
            "app/fetch/fetch.go",
            "package fetch\n\nimport \"os/exec\"\n\nfunc Run() { exec.Command(\"true\") }\n",
        ),
    ]);

    let run = scan(&repository, Tools::Missing, &[]);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(run.stdout.contains("\"go-exec-import\""), "{}", run.stdout);
}
