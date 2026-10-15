use serde_json::{Value, json};

use crate::support::TestRepository;

pub const APP_MODULE: &str = "app/go.mod";
pub const APP_MODULE_FILE: &str = "module example.com/app\n\ngo 1.22\n";
pub const FETCH_SOURCE: &str = "package fetch\n\nfunc Page(url string) error { return nil }\n";

pub struct Call<'a> {
    pub function: &'a str,
    pub package: &'a str,
    pub site: Option<(&'a str, u32)>,
}

pub struct Reach<'a> {
    pub package: &'a str,
    pub capability: &'a str,
    pub calls: &'a [Call<'a>],
}

pub fn capslock_output(reaches: &[Reach]) -> String {
    let infos: Vec<Value> = reaches
        .iter()
        .map(|reach| {
            let path: Vec<Value> = reach
                .calls
                .iter()
                .map(|call| {
                    let mut function = json!({ "name": call.function, "package": call.package });
                    if let Some((file, line)) = call.site {
                        function["site"] =
                            json!({ "filename": file, "line": line.to_string(), "column": "2" });
                    }
                    function
                })
                .collect();
            json!({
                "packageName": reach.package.rsplit('/').next(),
                "capabilityName": reach.capability,
                "capability": format!("CAPABILITY_{}", reach.capability.split('/').next().unwrap_or_default()),
                "path": path,
                "packageDir": reach.package,
            })
        })
        .collect();
    json!({ "capabilityInfo": infos, "moduleInfo": [], "packageInfo": [] }).to_string()
}

pub const FETCH_PACKAGE: &str = "example.com/app/fetch";
pub const PAGE: Call = Call {
    function: "example.com/app/fetch.Page",
    package: FETCH_PACKAGE,
    site: None,
};
pub const HTTP_GET: Call = Call {
    function: "net/http.Get",
    package: "net/http",
    site: Some(("fetch.go", 9)),
};
pub const RUN_TOOL: Call = Call {
    function: "example.com/dep/run.Tool",
    package: "example.com/dep/run",
    site: Some(("fetch.go", 6)),
};
pub const EXEC_COMMAND: Call = Call {
    function: "os/exec.Command",
    package: "os/exec",
    site: Some(("run.go", 5)),
};
pub const DIFF: &[&str] = &[
    "diff", "--base", "main", "--head", "feature", "--format", "json",
];

pub fn network_reaches() -> Vec<Reach<'static>> {
    ["example.com/app/fetch", "net/http"]
        .into_iter()
        .map(|package| Reach {
            package,
            capability: "NETWORK",
            calls: &[PAGE, HTTP_GET],
        })
        .collect()
}

pub fn exec_reaches() -> Vec<Reach<'static>> {
    ["example.com/app/fetch", "example.com/dep/run", "os/exec"]
        .into_iter()
        .map(|package| Reach {
            package,
            capability: "EXEC",
            calls: &[PAGE, RUN_TOOL, EXEC_COMMAND],
        })
        .collect()
}

pub fn module_repository(reaches: &[Reach]) -> TestRepository {
    let repository = TestRepository::new();
    repository.write(&[
        (APP_MODULE, APP_MODULE_FILE),
        ("app/fetch/fetch.go", FETCH_SOURCE),
        ("app/capslock.json", &capslock_output(reaches)),
    ]);
    repository
}

pub fn finding_summaries(report: &Value) -> Vec<String> {
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

pub const DIRECT_EXEC: Call = Call {
    function: "os/exec.Command",
    package: "os/exec",
    site: Some(("fetch.go", 4)),
};

pub fn direct_exec_reaches() -> Vec<Reach<'static>> {
    [FETCH_PACKAGE, "os/exec"]
        .into_iter()
        .map(|package| Reach {
            package,
            capability: "EXEC",
            calls: &[PAGE, DIRECT_EXEC],
        })
        .collect()
}
