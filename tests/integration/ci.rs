use crate::support::{expected, scan_files, sightings};

const WORKFLOWS: &str = ".github/workflows";

const RELEASE_WORKFLOW: &str = "name: release
on:
  pull_request_target:
    types: [opened]
  push:
permissions:
  contents: write
  id-token: write
  pull-requests: read
jobs:
  publish:
    permissions:
      packages: write
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: ./.github/actions/setup
      - uses: softprops/action-gh-release@v2
        with:
          token: ${{ secrets.NPM_TOKEN }}
          github: ${{ secrets.GITHUB_TOKEN }}
  reuse:
    uses: other-org/workflows/.github/workflows/build.yml@main
    secrets: inherit
";

#[test]
fn workflow_powers_are_found_with_their_lines() {
    let map = scan_files(&[(".github/workflows/release.yml", RELEASE_WORKFLOW)]);

    let file = ".github/workflows/release.yml";
    assert_eq!(
        sightings(&map),
        expected(&[
            &format!("ci {file} untrusted_trigger:pull_request_target {file}:3"),
            &format!("ci {file} token_write:contents {file}:7"),
            &format!("ci {file} oidc {file}:8"),
            &format!("ci {file} token_write:packages {file}:13"),
            &format!("ci {WORKFLOWS} third_party_action:softprops/action-gh-release {file}:18"),
            &format!("ci {file} secret:NPM_TOKEN {file}:20"),
            &format!(
                "ci {WORKFLOWS} third_party_action:other-org/workflows/.github/workflows/build.yml {file}:23"
            ),
            &format!("ci {file} secret:inherit {file}:24"),
        ])
    );
}

#[test]
fn trigger_lists_and_write_all_are_found() {
    let map = scan_files(&[
        (
            ".github/workflows/follow.yaml",
            "on: [push, workflow_run]\npermissions: write-all\njobs: {}\n",
        ),
        (
            ".github/workflows/target.yml",
            "on: pull_request_target\njobs: {}\n",
        ),
    ]);

    let follow = ".github/workflows/follow.yaml";
    let target = ".github/workflows/target.yml";
    assert_eq!(
        sightings(&map),
        expected(&[
            &format!("ci {follow} untrusted_trigger:workflow_run {follow}:1"),
            &format!("ci {follow} token_write:all {follow}:2"),
            &format!("ci {target} untrusted_trigger:pull_request_target {target}:1"),
        ])
    );
}

#[test]
fn a_read_only_workflow_has_no_powers() {
    let map = scan_files(&[(
        ".github/workflows/test.yml",
        "on: [push, pull_request]\npermissions:\n  contents: read\njobs:\n  test:\n    steps:\n      - uses: actions/checkout@v4\n        with:\n          token: ${{ secrets.GITHUB_TOKEN }}\n",
    )]);

    assert!(map.is_empty(), "{:?}", sightings(&map));
}

#[test]
fn github_expressions_inside_flow_lists_do_not_stop_the_scan() {
    let file = ".github/workflows/matrix.yml";
    let map = scan_files(&[(
        file,
        "on: push\njobs:\n  test:\n    strategy:\n      matrix:\n        python-version: [${{ env.VERSIONS }}]\n        include: [{os: linux}]\n    steps:\n      - run: echo ${{ secrets.DEPLOY_KEY }}\n",
    )]);

    assert_eq!(
        sightings(&map),
        expected(&[&format!("ci {file} secret:DEPLOY_KEY {file}:9")])
    );
}

#[test]
fn a_secret_is_read_only_from_an_expression_and_on_its_own_line() {
    let file = ".github/workflows/deploy.yml";
    let map = scan_files(&[(
        file,
        "on: push\njobs:\n  deploy:\n    steps:\n      - run: |\n          kubectl apply -f deploy/secrets.yaml\n          echo \"${{ secrets.DEPLOY_TOKEN }}\" | docker login\n",
    )]);

    assert_eq!(
        sightings(&map),
        expected(&[&format!("ci {file} secret:DEPLOY_TOKEN {file}:7")])
    );
}

#[test]
fn an_action_is_one_power_for_all_workflows() {
    let build = ".github/workflows/build.yml";
    let deploy = ".github/workflows/deploy.yml";
    let map = scan_files(&[
        (
            build,
            "on: push\njobs:\n  image:\n    steps:\n      - uses: docker/login-action@v3\n",
        ),
        (
            deploy,
            "on: push\njobs:\n  ship:\n    steps:\n      - uses: docker/login-action@v3\n      - uses: webfactory/ssh-agent@v0.9.0\n",
        ),
    ]);

    assert_eq!(
        sightings(&map),
        expected(&[
            &format!("ci {WORKFLOWS} third_party_action:docker/login-action {build}:5"),
            &format!("ci {WORKFLOWS} third_party_action:webfactory/ssh-agent {deploy}:6"),
        ])
    );
}

#[test]
fn actions_of_a_trusted_owner_are_first_party() {
    let file = ".github/workflows/release.yml";
    let map = scan_files(&[
        ("vigilante.toml", "[ci]\ntrusted_owners = [\"Acme\"]\n"),
        (
            file,
            "on: push\njobs:\n  release:\n    steps:\n      - uses: acme/setup-node@v1\n      - uses: acme-labs/publish@v2\n",
        ),
    ]);

    assert_eq!(
        sightings(&map),
        expected(&[&format!(
            "ci {WORKFLOWS} third_party_action:acme-labs/publish {file}:6"
        )])
    );
}
