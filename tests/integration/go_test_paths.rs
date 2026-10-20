use crate::go_fixtures::{
    APP_MODULE, APP_MODULE_FILE, Call, FETCH_PACKAGE, FETCH_SOURCE, PAGE, Reach, capslock_output,
};
use crate::go_support::{FakeTools, GoRun, Tools, run_vigilante};
use crate::support::TestRepository;

const HELPER_PACKAGE: &str = "example.com/app/tests/testutils/otel";
const HELPER_SOURCE: &str = "package otel\n\nfunc WaitForReady() error { return nil }\n";
const WAIT_FOR_READY: Call = Call {
    function: "example.com/app/tests/testutils/otel.WaitForReady",
    package: HELPER_PACKAGE,
    site: None,
};
const CALLED_FROM_FETCH: Call = Call {
    function: "example.com/app/tests/testutils/otel.WaitForReady",
    package: HELPER_PACKAGE,
    site: Some(("fetch.go", 3)),
};
const HELPER_EXEC: Call = Call {
    function: "os/exec.Command",
    package: "os/exec",
    site: Some(("collector.go", 3)),
};

fn scan(reaches: &[Reach]) -> crate::support::Run {
    let tools = FakeTools::new();
    let repository = TestRepository::new();
    repository.write(&[
        (APP_MODULE, APP_MODULE_FILE),
        ("app/fetch/fetch.go", FETCH_SOURCE),
        ("app/tests/testutils/otel/collector.go", HELPER_SOURCE),
        ("app/capslock.json", &capslock_output(reaches)),
    ]);
    run_vigilante(GoRun {
        repository: &repository,
        tools: &tools,
        available: Tools::Fake,
        arguments: &["scan", "--format", "json"],
        environment: &[],
    })
}

#[test]
fn exec_in_a_helper_under_a_tests_folder_is_not_a_finding() {
    let reaches = [Reach {
        package: HELPER_PACKAGE,
        capability: "EXEC",
        calls: &[WAIT_FOR_READY, HELPER_EXEC],
    }];

    let run = scan(&reaches);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(!run.stdout.contains("tests/testutils"), "{}", run.stdout);
}

#[test]
fn exec_in_production_code_that_calls_a_test_helper_stays_a_finding() {
    let reaches = [Reach {
        package: FETCH_PACKAGE,
        capability: "EXEC",
        calls: &[PAGE, CALLED_FROM_FETCH, HELPER_EXEC],
    }];

    let run = scan(&reaches);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(run.stdout.contains("app/fetch/fetch.go"), "{}", run.stdout);
    assert!(run.stdout.contains("\"exec\""), "{}", run.stdout);
}
