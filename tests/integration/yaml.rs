use std::fs;

use vigilante::{ScanRequest, Source};

use crate::support::{expected, scan_files, sightings, write_files};

const WORKFLOW: &str = ".github/workflows/matrix.yml";
const DEDENTED_FLOW_LISTS: &str = "on: pull_request
jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        version: [
          \"kindest/node:v1.31.9\",
          \"kindest/node:v1.32.5\"
        ]
        os: [
        ubuntu-latest,
        macos-latest
        ]
    steps:
      - run: |
          cat <<EOF
          [
          EOF
      - run: echo ${{ secrets.DEPLOY_KEY }}
    permissions:
      contents: write
";

#[test]
fn a_workflow_that_github_accepts_with_dedented_flow_lists_is_read() {
    let map = scan_files(&[(WORKFLOW, DEDENTED_FLOW_LISTS)]);

    assert_eq!(
        sightings(&map),
        expected(&[
            &format!("ci {WORKFLOW} token_write:contents {WORKFLOW}:22"),
            &format!("ci {WORKFLOW} secret:DEPLOY_KEY {WORKFLOW}:20"),
        ])
    );
}

#[test]
fn a_workflow_with_an_unclosed_flow_list_still_stops_the_scan() {
    let directory = write_files(&[(WORKFLOW, "on: [push\njobs: {}\n")]);

    let result = vigilante::scan(ScanRequest {
        repository: directory.path().to_path_buf(),
        source: Source::WorkingDirectory,
    });

    let error = result.expect_err("an unclosed flow list is invalid YAML");
    assert!(error.to_string().contains("cannot parse"), "{error}");
    assert!(fs::metadata(directory.path().join(WORKFLOW)).is_ok());
}
