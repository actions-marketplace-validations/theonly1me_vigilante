use serde_json::Value;

use crate::support::TestRepository;

const APP_MODULE: &str = "module example.com/app\n\ngo 1.22\n\nrequire example.com/dep v0.0.0\n\nreplace example.com/dep => ../dep\n";
const DEP_MODULE: &str = "module example.com/dep\n\ngo 1.22\n";
const RUN_SOURCE: &str = "package run\n\nimport \"os/exec\"\n\nfunc Tool(name string) error {\n\treturn exec.Command(name).Run()\n}\n";
const FETCH_AT_BASE: &str = "package fetch\n\nimport \"net/http\"\n\nfunc Page(url string) error {\n\t_, err := http.Get(url)\n\treturn err\n}\n";
const FETCH_AT_HEAD: &str = "package fetch\n\nimport (\n\t\"net/http\"\n\n\t\"example.com/dep/run\"\n)\n\nfunc Page(url string) error {\n\tif err := run.Tool(\"true\"); err != nil {\n\t\treturn err\n\t}\n\t_, err := http.Get(url)\n\treturn err\n}\n";

#[test]
#[ignore = "needs go and capslock on PATH"]
fn real_capslock_finds_exec_that_a_package_reaches_through_a_dependency() {
    let repository = TestRepository::new();
    repository.write(&[
        ("app/go.mod", APP_MODULE),
        ("app/fetch/fetch.go", FETCH_AT_BASE),
        ("dep/go.mod", DEP_MODULE),
        ("dep/run/run.go", RUN_SOURCE),
    ]);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(&[("app/fetch/fetch.go", FETCH_AT_HEAD)]);
    repository.commit("run a tool");

    let run = repository.vigilante(&[
        "diff", "--base", "main", "--head", "feature", "--format", "json",
    ]);

    let report: Value =
        serde_json::from_str(&run.stdout).unwrap_or_else(|_| panic!("{}", run.stderr));
    let findings: Vec<String> = report["findings"]
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
        .collect();
    assert_eq!(findings, ["app/fetch exec app/fetch/fetch.go:10"]);
    let call_path = report["findings"][0]["evidence"]["call_path"].to_string();
    assert!(
        call_path.contains("example.com/dep/run.Tool"),
        "{call_path}"
    );
    assert!(call_path.contains("os/exec."), "{call_path}");
    let packages: Vec<&str> = report["notes"]
        .as_array()
        .expect("notes")
        .iter()
        .filter_map(|note| note["package"].as_str())
        .collect();
    assert_eq!(packages, ["example.com/dep/run", "os/exec"]);
    assert_eq!(run.code, 1);
}
