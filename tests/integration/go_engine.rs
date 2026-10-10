use crate::go_fixtures::{
    DIFF, Reach, capslock_output, direct_exec_reaches, exec_reaches, finding_summaries,
    module_repository, network_reaches,
};
use crate::go_support::{FakeTools, GoRun, Tools, report_of, run_vigilante};
#[test]
fn diff_reports_a_capability_that_a_go_package_gains_at_the_head() {
    let tools = FakeTools::new();
    let repository = module_repository(&network_reaches());
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    let both: Vec<Reach> = network_reaches()
        .into_iter()
        .chain(direct_exec_reaches())
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
        ["app/fetch exec app/fetch/fetch.go:4"]
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
        .chain(direct_exec_reaches())
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
