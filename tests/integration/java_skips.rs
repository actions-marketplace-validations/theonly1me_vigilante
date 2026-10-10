use crate::npm_registry::report_of;
use crate::support::TestRepository;

const DIFF: &[&str] = &[
    "diff", "--base", "main", "--head", "feature", "--format", "json",
];
const MAIN: &str = "app/src/main/java/com/acme/app/Main.java";
const EMPTY_MAIN: &str =
    "package com.acme.app;\n\npublic class Main {\n    public void go() {}\n}\n";
const RUNNING_MAIN: &str = "package com.acme.app;\n\nimport com.acme.runner.Runner;\n\npublic class Main {\n    public void go() {\n        Runner.start();\n    }\n}\n";
const PRIVATE_REPOSITORY: &str = "https://maven.acme.dev/releases";

fn reasons_after(build: (&str, &str)) -> Vec<String> {
    let repository = TestRepository::new();
    repository.write(&[build, (MAIN, EMPTY_MAIN)]);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(&[(MAIN, RUNNING_MAIN)]);
    repository.commit("start the runner");
    let report = report_of(&repository.run_with_cache(DIFF));
    report["skipped"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|skip| skip["reason"].as_str().map(str::to_string))
        .collect()
}

#[test]
fn a_repository_that_is_not_allowed_is_listed() {
    let pom = format!(
        "<project><groupId>com.acme</groupId><artifactId>app</artifactId><version>1</version><repositories><repository><url>{PRIVATE_REPOSITORY}</url></repository></repositories></project>"
    );

    let reasons = reasons_after(("app/pom.xml", &pom));

    assert_eq!(
        reasons,
        vec![format!(
            "The repository `{PRIVATE_REPOSITORY}` is not public. Add it to `[registries] allow` in vigilante.toml if it needs no credentials."
        )]
    );
}

#[test]
fn a_gradle_build_without_a_lockfile_is_listed() {
    let reasons = reasons_after(("app/build.gradle", "plugins { id 'java' }\n"));

    assert_eq!(
        reasons,
        vec![
            "No gradle.lockfile covers this code. Vigilante does not run Gradle, so it read only the code in the repository."
        ]
    );
}

#[test]
fn a_pom_that_is_not_valid_is_listed() {
    let pom = "<project><groupId>com.acme</groupId><artifactId>app</artifactId><version>1</version></project>\n\n<dependency><groupId>com.acme</groupId></dependency>\n</dependencies>\n</project>\n";

    let reasons = reasons_after(("app/pom.xml", pom));

    assert_eq!(
        reasons,
        vec![
            "This pom.xml is not a valid Maven project file, so Vigilante read only the code in the repository."
        ]
    );
}
