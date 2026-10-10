use crate::support::{expected, scan_files, sightings};

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
            &format!("ci {file} third_party_action:softprops/action-gh-release {file}:18"),
            &format!("ci {file} secret:NPM_TOKEN {file}:20"),
            &format!(
                "ci {file} third_party_action:other-org/workflows/.github/workflows/build.yml {file}:23"
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
