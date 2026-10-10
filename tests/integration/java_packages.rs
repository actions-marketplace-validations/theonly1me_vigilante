use std::collections::BTreeMap;

use crate::file_server::{allow_config, serve_files};
use crate::java_classes::{Artifact, TestCall, TestClass, TestMethod, artifact_files};
use crate::npm_registry::report_of;
use crate::python_fixtures::summaries;
use crate::support::{Run, TestRepository};

const DIFF: &[&str] = &[
    "diff", "--base", "main", "--head", "feature", "--format", "json",
];
const MAIN: &str = "app/src/main/java/com/acme/app/Main.java";
const EMPTY_MAIN: &str =
    "package com.acme.app;\n\npublic class Main {\n    public void go() {}\n}\n";
const RUNNING_MAIN: &str = "package com.acme.app;\n\nimport com.acme.runner.Runner;\n\npublic class Main {\n    public void go() {\n        Runner.start();\n    }\n}\n";
const GRADLE_LOCKFILE: &str = "com.acme:runner:1.0=compileClasspath,runtimeClasspath\ncom.acme:spawner:2.0=runtimeClasspath\nempty=annotationProcessor\n";
const RUNNER: TestClass = TestClass {
    name: "com/acme/runner/Runner",
    methods: &[TestMethod {
        name: "start",
        calls: &[TestCall {
            class: "com/acme/spawner/Spawner",
            name: "spawn",
            is_static: true,
        }],
    }],
};
const SPAWNER: TestClass = TestClass {
    name: "com/acme/spawner/Spawner",
    methods: &[TestMethod {
        name: "spawn",
        calls: &[TestCall {
            class: "java/lang/ProcessBuilder",
            name: "<init>",
            is_static: false,
        }],
    }],
};

fn app_pom(repository: &str) -> String {
    format!(
        "<project><groupId>com.acme</groupId><artifactId>app</artifactId><version>1</version>\n<repositories><repository><url>{repository}</url></repository></repositories>\n<dependencies><dependency><groupId>com.acme</groupId><artifactId>runner</artifactId><version>1.0</version></dependency></dependencies></project>\n"
    )
}

struct Setup<'a> {
    gradle: bool,
    sha1_of_runner: Option<&'a str>,
}

fn served_repository(setup: Setup) -> (TestRepository, String) {
    let mut files = BTreeMap::new();
    files.extend(artifact_files(Artifact {
        name: "runner",
        version: "1.0",
        pom: "<project><groupId>com.acme</groupId><artifactId>runner</artifactId><version>1.0</version><dependencies><dependency><groupId>com.acme</groupId><artifactId>spawner</artifactId><version>2.0</version></dependency></dependencies></project>",
        classes: &[&RUNNER],
    }));
    files.extend(artifact_files(Artifact {
        name: "spawner",
        version: "2.0",
        pom: "<project><groupId>com.acme</groupId><artifactId>spawner</artifactId><version>2.0</version></project>",
        classes: &[&SPAWNER],
    }));
    if let Some(sha1) = setup.sha1_of_runner {
        files.insert(
            "/com/acme/runner/1.0/runner-1.0.jar.sha1".to_string(),
            sha1.as_bytes().to_vec(),
        );
    }
    let base_url = serve_files(files);
    let config = allow_config(&base_url);
    let build = if setup.gradle {
        vec![
            ("app/build.gradle", "plugins { id 'java' }\n".to_string()),
            ("app/gradle.lockfile", GRADLE_LOCKFILE.to_string()),
        ]
    } else {
        vec![("app/pom.xml", app_pom(&base_url))]
    };
    let repository = TestRepository::new();
    for (path, text) in &build {
        repository.write(&[(path, text)]);
    }
    repository.write(&[("vigilante.toml", &config), (MAIN, EMPTY_MAIN)]);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(&[(MAIN, RUNNING_MAIN)]);
    repository.commit("start the runner");
    (repository, base_url)
}

#[test]
fn a_new_maven_dependency_that_runs_programs_is_a_note_with_its_call_path() {
    let (repository, _) = served_repository(Setup {
        gradle: false,
        sha1_of_runner: None,
    });

    let run = repository.run_with_cache(DIFF);

    assert_notes_follow_the_runner(&run);
}

#[test]
fn a_new_gradle_dependency_is_read_from_the_lockfile_and_an_allowed_registry() {
    let (repository, _) = served_repository(Setup {
        gradle: true,
        sha1_of_runner: None,
    });

    let run = repository.run_with_cache(DIFF);

    assert_notes_follow_the_runner(&run);
}

fn assert_notes_follow_the_runner(run: &Run) {
    let report = report_of(run);
    let path = "app/src/main/java/com/acme/app/Main.Main.go > com.acme:runner/com/acme/runner/Runner.Runner.start > com.acme:spawner/com/acme/spawner/Spawner.Spawner.spawn > new ProcessBuilder";
    assert_eq!(
        summaries(&report["findings"]),
        Vec::<String>::new(),
        "{}",
        run.stderr
    );
    let notes = summaries(&report["notes"]);
    assert!(
        notes.contains(&format!("app exec com.acme:runner {MAIN}:7 {path}")),
        "{notes:#?}\n{}",
        run.stderr
    );
    assert!(
        notes.contains(&format!("app exec com.acme:spawner {MAIN}:7 {path}")),
        "{notes:#?}"
    );
}

#[test]
fn a_jar_that_does_not_match_its_published_sha1_stops_the_review() {
    let (repository, _) = served_repository(Setup {
        gradle: false,
        sha1_of_runner: Some("0000000000000000000000000000000000000000"),
    });

    let run = repository.run_with_cache(DIFF);

    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(
        run.stderr.contains("does not match the checksum"),
        "{}",
        run.stderr
    );
}
