use crate::go_fixtures::{
    APP_MODULE, APP_MODULE_FILE, FETCH_SOURCE, capslock_output, network_reaches,
};
use crate::go_support::{FakeTools, GoRun, Tools, run_vigilante};
use crate::support::TestRepository;

const DOCS_MODULE: &str = "docs/go.mod";
const DOCS_MODULE_FILE: &str = "module example.com/docs\n\ngo 1.22\n";

fn scan(repository: &TestRepository, tools: &FakeTools, available: Tools) -> crate::support::Run {
    run_vigilante(GoRun {
        repository,
        tools,
        available,
        arguments: &["scan", "--format", "json"],
        environment: &[],
    })
}

#[test]
fn a_module_folder_without_go_files_is_not_analyzed() {
    let tools = FakeTools::new();
    let repository = TestRepository::new();
    repository.write(&[
        (APP_MODULE, APP_MODULE_FILE),
        ("app/fetch/fetch.go", FETCH_SOURCE),
        ("app/capslock.json", &capslock_output(&network_reaches())),
        (DOCS_MODULE, DOCS_MODULE_FILE),
        ("docs/config.toml", "title = \"docs\"\n"),
    ]);

    let run = scan(&repository, &tools, Tools::Fake);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(run.stdout.contains("app/fetch"), "{}", run.stdout);
    assert!(run.stdout.contains("\"network\""), "{}", run.stdout);
    let calls: Vec<String> = tools.log().lines().map(String::from).collect();
    assert_eq!(calls.len(), 1, "{calls:?}");
    assert!(calls[0].contains("/app "), "{calls:?}");
}

#[test]
fn a_repository_with_only_modules_without_go_files_needs_no_go_tools() {
    let tools = FakeTools::new();
    let repository = TestRepository::new();
    repository.write(&[
        (DOCS_MODULE, DOCS_MODULE_FILE),
        ("docs/config.toml", "title = \"docs\"\n"),
    ]);

    let run = scan(&repository, &tools, Tools::Missing);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(tools.log(), "");
}

#[test]
fn a_go_file_in_a_nested_module_does_not_count_for_the_outer_module() {
    let tools = FakeTools::new();
    let repository = TestRepository::new();
    repository.write(&[
        ("go.mod", DOCS_MODULE_FILE),
        (APP_MODULE, APP_MODULE_FILE),
        ("app/fetch/fetch.go", FETCH_SOURCE),
        ("app/capslock.json", &capslock_output(&network_reaches())),
    ]);

    let run = scan(&repository, &tools, Tools::Fake);

    assert_eq!(run.code, 0, "{}", run.stderr);
    let calls: Vec<String> = tools.log().lines().map(String::from).collect();
    assert_eq!(calls.len(), 1, "{calls:?}");
}

#[test]
#[ignore = "needs go and capslock on PATH"]
fn real_capslock_scans_go_code_next_to_a_module_without_go_files() {
    let repository = TestRepository::new();
    repository.write(&[
        (APP_MODULE, APP_MODULE_FILE),
        (
            "app/run/run.go",
            "package run\n\nimport \"os/exec\"\n\nfunc Tool() error {\n\treturn exec.Command(\"true\").Run()\n}\n",
        ),
        (DOCS_MODULE, DOCS_MODULE_FILE),
    ]);

    let run = repository.vigilante(&["scan", "--format", "json"]);

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(run.stdout.contains("app/run"), "{}", run.stdout);
}
