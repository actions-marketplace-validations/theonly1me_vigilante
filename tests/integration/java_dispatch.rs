use std::collections::BTreeMap;

use crate::file_server::{allow_config, serve_files};
use crate::java_classes::{Artifact, TestCall, TestClass, TestMethod, artifact_files};
use crate::npm_registry::report_of;
use crate::python_fixtures::summaries;
use crate::support::TestRepository;

const DIFF: &[&str] = &[
    "diff", "--base", "main", "--head", "feature", "--format", "json",
];
const MAIN: &str = "app/src/main/java/com/acme/app/Main.java";
const EMPTY_MAIN: &str =
    "package com.acme.app;\n\npublic class Main {\n    public void go() {}\n}\n";
const CALLING_MAIN: &str = "package com.acme.app;\n\nimport com.acme.api.Service;\n\npublic class Main {\n    public void go() {\n        Service.create().init();\n    }\n}\n";
const LOADING_MAIN: &str = "package com.acme.app;\n\nimport com.acme.api.Loader;\n\npublic class Main {\n    public void go() {\n        Loader.ready();\n    }\n}\n";
const SERVICE: TestClass = TestClass {
    name: "com/acme/api/Service",
    interfaces: &[],
    methods: &[
        TestMethod {
            name: "init",
            calls: &[TestCall {
                class: "java/net/Socket",
                name: "<init>",
                is_static: false,
            }],
        },
        TestMethod {
            name: "create",
            calls: &[],
        },
        TestMethod {
            name: "start",
            calls: &[TestCall {
                class: "com/acme/api/Loader",
                name: "ready",
                is_static: true,
            }],
        },
    ],
};
const TOOL: TestClass = TestClass {
    name: "com/acme/api/Tool",
    interfaces: &[],
    methods: &[TestMethod {
        name: "run",
        calls: &[],
    }],
};
const SHELL_TOOL: TestClass = TestClass {
    name: "com/acme/api/ShellTool",
    interfaces: &["com/acme/api/Tool"],
    methods: &[
        TestMethod {
            name: "<init>",
            calls: &[],
        },
        TestMethod {
            name: "run",
            calls: &[TestCall {
                class: "java/lang/ProcessBuilder",
                name: "<init>",
                is_static: false,
            }],
        },
    ],
};
const TOOLS: TestClass = TestClass {
    name: "com/acme/api/Tools",
    interfaces: &[],
    methods: &[TestMethod {
        name: "create",
        calls: &[TestCall {
            class: "com/acme/api/ShellTool",
            name: "<init>",
            is_static: false,
        }],
    }],
};
const TOOL_MAIN: &str = "package com.acme.app;\n\nimport com.acme.api.Tool;\nimport com.acme.api.Tools;\n\npublic class Main {\n    public void go() {\n        Tool tool = Tools.create();\n        tool.run();\n    }\n}\n";
const LOADER: TestClass = TestClass {
    name: "com/acme/api/Loader",
    interfaces: &[],
    methods: &[
        TestMethod {
            name: "<clinit>",
            calls: &[TestCall {
                class: "java/lang/ProcessBuilder",
                name: "<init>",
                is_static: false,
            }],
        },
        TestMethod {
            name: "ready",
            calls: &[],
        },
    ],
};

fn repository_calling(main: &str) -> TestRepository {
    repository_with(Setup {
        classes: &[&SERVICE, &LOADER],
        main,
    })
}

struct Setup<'a> {
    classes: &'a [&'a TestClass<'a>],
    main: &'a str,
}

fn repository_with(setup: Setup) -> TestRepository {
    let Setup { classes, main } = setup;
    let mut files = BTreeMap::new();
    files.extend(artifact_files(Artifact {
        name: "api",
        version: "1.0",
        pom: "<project><groupId>com.acme</groupId><artifactId>api</artifactId><version>1.0</version></project>",
        classes,
    }));
    let base_url = serve_files(files);
    let pom = format!(
        "<project><groupId>com.acme</groupId><artifactId>app</artifactId><version>1</version><repositories><repository><url>{base_url}</url></repository></repositories><dependencies><dependency><groupId>com.acme</groupId><artifactId>api</artifactId><version>1.0</version></dependency></dependencies></project>"
    );
    let repository = TestRepository::new();
    repository.write(&[
        ("app/pom.xml", &pom),
        ("vigilante.toml", &allow_config(&base_url)),
        (MAIN, EMPTY_MAIN),
    ]);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(&[(MAIN, main)]);
    repository.commit("call the service");
    repository
}

#[test]
fn a_call_by_method_name_does_not_reach_the_top_level_code_of_other_files() {
    let repository = repository_calling(CALLING_MAIN);

    let run = repository.run_with_cache(DIFF);

    let report = report_of(&run);
    let notes = summaries(&report["notes"]);
    let note = format!(
        "app network com.acme:api {MAIN}:7 app/src/main/java/com/acme/app/Main.Main.go > com.acme:api/com/acme/api/Service.Service.init > java.net.Socket.constructor"
    );
    assert_eq!(notes, vec![note.clone(), note], "{}", run.stderr);
}

#[test]
fn a_call_to_a_static_method_runs_the_static_initializer_of_its_class() {
    let repository = repository_calling(LOADING_MAIN);

    let run = repository.run_with_cache(DIFF);

    let notes = summaries(&report_of(&run)["notes"]);
    let note = format!(
        "app exec com.acme:api {MAIN}:7 app/src/main/java/com/acme/app/Main.Main.go > com.acme:api/com/acme/api/Loader.Loader.ready > com.acme:api/com/acme/api/Loader.init > new ProcessBuilder"
    );
    assert_eq!(notes, vec![note.clone(), note], "{}", run.stderr);
}

#[test]
fn a_call_on_an_interface_reaches_the_loaded_classes_that_implement_it() {
    let repository = repository_with(Setup {
        classes: &[&TOOL, &SHELL_TOOL, &TOOLS],
        main: TOOL_MAIN,
    });

    let run = repository.run_with_cache(DIFF);

    let notes = summaries(&report_of(&run)["notes"]);
    let note = format!(
        "app exec com.acme:api {MAIN}:9 app/src/main/java/com/acme/app/Main.Main.go > com.acme:api/com/acme/api/ShellTool.ShellTool.run > new ProcessBuilder"
    );
    assert_eq!(notes, vec![note.clone(), note], "{}", run.stderr);
}
