use std::time::Duration;

use super::{ContainerRequest, ContainerRun, run_contained};

const IMAGE: &str =
    "busybox:1.37@sha256:bdf57e528e45e4433820e045b29b4597825a1c9e38353532d90a01445013f82e";
const FETCH: &str = "wget -q -T 10 -O /dev/null http://example.com";

fn run(build: &str) -> ContainerRun {
    let source = tempfile::tempdir().expect("source");
    let output = tempfile::tempdir().expect("output");
    run_contained(ContainerRequest {
        image: IMAGE,
        source: &std::fs::canonicalize(source.path()).expect("source path"),
        output: &std::fs::canonicalize(output.path()).expect("output path"),
        prepare: FETCH,
        build,
        timeout: Duration::from_secs(120),
    })
    .expect("docker runs")
}

#[test]
#[ignore = "needs docker"]
fn docker_the_build_step_has_no_network_after_the_prepare_step_had_it() {
    let built = run(&format!("! {FETCH}"));

    assert!(
        matches!(built, ContainerRun::Built),
        "the build step reached the network"
    );
}

#[test]
#[ignore = "needs docker"]
fn docker_a_failed_build_step_is_not_built() {
    let built = run("exit 3");

    assert!(matches!(built, ContainerRun::NotBuilt(_)));
}
