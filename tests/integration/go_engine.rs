use crate::go_support::{
    APP_MODULE, APP_MODULE_FILE, Call, FETCH_SOURCE, FakeTools, GoRun, Reach, Tools,
    capslock_output, report_of, run_vigilante,
};
use crate::support::TestRepository;

const FETCH_PACKAGE: &str = "example.com/app/fetch";
const PAGE: Call = Call {
    function: "example.com/app/fetch.Page",
    package: FETCH_PACKAGE,
    site: None,
};
const HTTP_GET: Call = Call {
    function: "net/http.Get",
    package: "net/http",
    site: Some(("fetch.go", 9)),
};
const RUN_TOOL: Call = Call {
    function: "example.com/dep/run.Tool",
    package: "example.com/dep/run",
    site: Some(("fetch.go", 6)),
};
const EXEC_COMMAND: Call = Call {
    function: "os/exec.Command",
    package: "os/exec",
    site: Some(("run.go", 5)),
};
const DIFF: &[&str] = &[
    "diff", "--base", "main", "--head", "feature", "--format", "json",
];

fn network_reaches() -> Vec<Reach<'static>> {
    ["example.com/app/fetch", "net/http"]
        .into_iter()
        .map(|package| Reach {
            package,
            capability: "NETWORK",
            calls: &[PAGE, HTTP_GET],
        })
        .collect()
}

fn exec_reaches() -> Vec<Reach<'static>> {
    ["example.com/app/fetch", "example.com/dep/run", "os/exec"]
        .into_iter()
        .map(|package| Reach {
            package,
            capability: "EXEC",
            calls: &[PAGE, RUN_TOOL, EXEC_COMMAND],
        })
        .collect()
}

fn module_repository(reaches: &[Reach]) -> TestRepository {
    let repository = TestRepository::new();
    repository.write(&[
        (APP_MODULE, APP_MODULE_FILE),
        ("app/fetch/fetch.go", FETCH_SOURCE),
        ("app/capslock.json", &capslock_output(reaches)),
    ]);
    repository
}

fn finding_summaries(report: &serde_json::Value) -> Vec<String> {
    report["findings"]
        .as_array()
        .expect("findings")
        .iter()
        .map(|finding| {
            format!(
                "{} {} {}:{}",
                finding["scope"]["path"].as_str().unwrap_or_default(),
                finding["power"].as_str().unwrap_or_default(),
                finding["evidence"]["path"].as_str().unwrap_or_default(),
                finding["evidence"]["line"]
            )
        })
        .collect()
}

#[test]
fn diff_reports_a_capability_that_a_go_package_gains_at_the_head() {
    let tools = FakeTools::new();
    let repository = module_repository(&network_reaches());
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    let both: Vec<Reach> = network_reaches()
        .into_iter()
        .chain(exec_reaches())
        .collect();
    repository.write(&[("app/capslock.json", &capslock_output(&both))]);
    repository.commit("run a tool");

    let run = run_vigilante(GoRun {
        repository: &repository,
        tools: &tools,
        available: Tools::Fake,
        arguments: DIFF,
        environment: &[],
    });

    assert_eq!(
        finding_summaries(&report_of(&run)),
        ["app/fetch exec app/fetch/fetch.go:6"]
    );
    assert_eq!(run.code, 1, "{}", run.stderr);
    let calls: Vec<String> = tools.log().lines().map(String::from).collect();
    assert_eq!(calls.len(), 2, "{calls:?}");
    assert!(calls.iter().all(|call| {
        call.ends_with("/app -packages=./... -output=json -granularity=intermediate")
    }));
}

#[test]
fn the_lock_records_the_capabilities_of_each_go_package() {
    let tools = FakeTools::new();
    let all: Vec<Reach> = network_reaches()
        .into_iter()
        .chain(exec_reaches())
        .collect();
    let repository = module_repository(&all);

    let run = run_vigilante(GoRun {
        repository: &repository,
        tools: &tools,
        available: Tools::Fake,
        arguments: &["lock"],
        environment: &[],
    });

    assert_eq!(run.code, 0, "{}", run.stderr);
    let lock =
        std::fs::read_to_string(repository.directory.path().join("vigilante.lock")).expect("lock");
    assert_eq!(
        lock,
        "version = 1\n\n[code.\"app/fetch\"]\npowers = [\n    \"exec\",\n    \"network\",\n]\n"
    );
}

#[test]
fn a_change_outside_go_modules_does_not_need_capslock() {
    let tools = FakeTools::new();
    let repository = module_repository(&network_reaches());
    repository.write(&[("README.md", "# app\n")]);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(&[("README.md", "# app\n\nMore words.\n")]);
    repository.commit("docs");

    let run = run_vigilante(GoRun {
        repository: &repository,
        tools: &tools,
        available: Tools::Missing,
        arguments: DIFF,
        environment: &[],
    });

    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(finding_summaries(&report_of(&run)).is_empty());
}

#[test]
fn a_package_of_another_go_module_is_reported_by_its_own_module() {
    let tools = FakeTools::new();
    let repository = module_repository(&network_reaches());
    repository.write(&[
        ("dep/go.mod", "module example.com/dep\n\ngo 1.22\n"),
        ("dep/run/run.go", "package run\n"),
        ("dep/capslock.json", &capslock_output(&[])),
    ]);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    let both: Vec<Reach> = network_reaches()
        .into_iter()
        .chain(exec_reaches())
        .collect();
    repository.write(&[("app/capslock.json", &capslock_output(&both))]);
    repository.commit("run a tool from the dep module");

    let run = run_vigilante(GoRun {
        repository: &repository,
        tools: &tools,
        available: Tools::Fake,
        arguments: DIFF,
        environment: &[],
    });

    assert_eq!(
        finding_summaries(&report_of(&run)),
        ["app/fetch exec app/fetch/fetch.go:6"]
    );
}
