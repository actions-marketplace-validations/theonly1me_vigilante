use serde_json::Value;

use crate::npm_registry::report_of;
use crate::python_fixtures::summaries;
use crate::support::TestRepository;

const DIFF: &[&str] = &[
    "diff", "--base", "main", "--head", "feature", "--format", "json",
];
const APP_POM: (&str, &str) = (
    "app/pom.xml",
    "<project><artifactId>app</artifactId></project>\n",
);
const MAIN: &str = "app/src/main/java/com/acme/app/Main.java";

struct Change<'a> {
    base: &'a [(&'a str, &'a str)],
    head: &'a [(&'a str, &'a str)],
}

fn diff(change: Change) -> Value {
    let repository = TestRepository::new();
    repository.write(&[
        APP_POM,
        (
            MAIN,
            "package com.acme.app;\n\npublic class Main {\n    public void go() {}\n}\n",
        ),
    ]);
    repository.write(change.base);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(change.head);
    repository.commit("change");
    report_of(&repository.run_with_cache(DIFF))
}

fn main_with(body: &str) -> String {
    format!("package com.acme.app;\n\n{body}")
}

#[test]
fn own_code_that_reaches_exec_through_another_module_is_a_finding() {
    let report = diff(Change {
        base: &[],
        head: &[
            (
                MAIN,
                &main_with(
                    "import com.acme.tools.Shell;\n\npublic class Main {\n    public void go() {\n        Shell shell = new Shell();\n        shell.run(\"make\");\n    }\n}\n",
                ),
            ),
            (
                "tools/pom.xml",
                "<project><artifactId>tools</artifactId></project>\n",
            ),
            (
                "tools/src/main/java/com/acme/tools/Shell.java",
                "package com.acme.tools;\n\npublic class Shell {\n    public void run(String command) throws Exception {\n        new ProcessBuilder(command).start();\n    }\n}\n",
            ),
        ],
    });

    let findings = summaries(&report["findings"]);

    assert!(
        findings.contains(&"app exec own app/src/main/java/com/acme/app/Main.java:8 app/src/main/java/com/acme/app/Main.Main.go > tools/src/main/java/com/acme/tools/Shell.Shell.run > new ProcessBuilder".to_string()),
        "{findings:#?}"
    );
}

#[test]
fn a_privileged_import_used_through_a_variable_is_a_finding() {
    let report = diff(Change {
        base: &[],
        head: &[(
            MAIN,
            &main_with(
                "import java.net.http.HttpClient;\n\npublic class Main {\n    public void go(HttpClient client) throws Exception {\n        client.send(null, null);\n    }\n}\n",
            ),
        )],
    });

    let findings = summaries(&report["findings"]);

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(
        findings[0].starts_with("app network own app/src/main/java/com/acme/app/Main.java:"),
        "{findings:#?}"
    );
}

#[test]
fn an_import_that_is_never_used_is_not_a_power() {
    let report = diff(Change {
        base: &[],
        head: &[(
            MAIN,
            &main_with(
                "import java.net.Socket;\n\npublic class Main {\n    public void go() {}\n}\n",
            ),
        )],
    });

    assert_eq!(summaries(&report["findings"]), Vec::<String>::new());
}

#[test]
fn a_native_method_is_a_finding() {
    let report = diff(Change {
        base: &[],
        head: &[(
            MAIN,
            &main_with("public class Main {\n    public native void attach();\n}\n"),
        )],
    });

    assert_eq!(
        summaries(&report["findings"]),
        vec![
            "app native own app/src/main/java/com/acme/app/Main.java:4 app/src/main/java/com/acme/app/Main.Main.attach > public native"
        ]
    );
}

#[test]
fn the_patterns_engine_reports_an_import_without_the_call_graph() {
    let report = diff(Change {
        base: &[("vigilante.toml", "[engines]\njava = \"patterns\"\n")],
        head: &[(
            MAIN,
            &main_with(
                "import java.net.Socket;\n\npublic class Main {\n    public void go() {}\n}\n",
            ),
        )],
    });

    assert_eq!(
        summaries(&report["findings"]),
        vec!["app network own app/src/main/java/com/acme/app/Main.java:3 "]
    );
}
