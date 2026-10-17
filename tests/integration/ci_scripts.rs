use crate::risks::{Change, review_of, risk_lines};
use crate::support::{expected, scan_files, sightings};

const PATH: &str = ".github/workflows/test.yml";

fn workflow_running(lines: &[&str]) -> String {
    let steps: String = lines
        .iter()
        .map(|line| format!("      - run: {line}\n"))
        .collect();
    format!("on: push\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n{steps}")
}

#[test]
fn only_downloads_that_an_interpreter_runs_are_remote_scripts() {
    let map = scan_files(&[(
        PATH,
        &workflow_running(&[
            "TAG=$(curl -s https://api.example.invalid/latest)",
            "curl -s https://api.example.invalid/tags | python3 -c 'import json'",
            "curl -s https://api.example.invalid/tags | jq .name",
            "bash <(curl -s https://first.example.invalid/install.sh)",
            "sh -c \"$(curl -fsSL https://second.example.invalid/install.sh)\"",
            "cd tools && curl -s https://third.example.invalid/x | sudo bash -s -- --yes",
        ]),
    )]);

    assert_eq!(
        sightings(&map),
        expected(&[
            &format!("ci .github/workflows remote_script:first.example.invalid {PATH}:9"),
            &format!("ci .github/workflows remote_script:second.example.invalid {PATH}:10"),
            &format!("ci .github/workflows remote_script:third.example.invalid {PATH}:11"),
        ])
    );
}

#[test]
fn the_secrets_word_inside_a_string_is_not_a_secret() {
    let map = scan_files(&[(
        PATH,
        &workflow_running(&[
            "echo '${{ contains(github.event.head_commit.message, 'secrets') }}'",
            "echo '${{ github.event.inputs.secrets }}'",
        ]),
    )]);

    assert_eq!(sightings(&map), expected(&[]));
}

#[test]
fn an_action_from_a_trusted_owner_has_no_pin_risk() {
    let pinned = "vendor/action@0123456789abcdef0123456789abcdef01234567";
    let lines = risk_lines(&review_of(Change {
        base: &[
            ("vigilante.toml", "[ci]\ntrusted_owners = [\"vendor\"]\n"),
            (
                PATH,
                &format!("on: push\njobs:\n  test:\n    uses: {pinned}\n"),
            ),
        ],
        head: &[(
            PATH,
            "on: push\njobs:\n  test:\n    uses: vendor/action@main\n",
        )],
    }));

    assert_eq!(lines, Vec::<String>::new());
}

#[test]
fn bracket_access_and_serialized_secrets_are_secrets() {
    let path = ".github/workflows/test.yml";
    let map = scan_files(&[(
        path,
        "on: push\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - run: |\n          echo \"${{ secrets['DEPLOY_TOKEN'] }}\"\n      - run: echo '${{ toJSON(secrets) }}'\n",
    )]);

    assert_eq!(
        sightings(&map),
        expected(&[
            &format!("ci {path} secret:DEPLOY_TOKEN {path}:7"),
            &format!("ci {path} secret:all {path}:8"),
        ])
    );
}

#[test]
fn a_composite_action_that_runs_a_downloaded_script_is_a_remote_script() {
    let map = scan_files(&[(
        ".github/actions/setup/action.yml",
        "name: setup\nruns:\n  using: composite\n  steps:\n    - shell: bash\n      run: curl https://example.invalid/install.sh | sh\n    - uses: vendor/setup@v1\n",
    )]);

    let path = ".github/actions/setup/action.yml";
    assert_eq!(
        sightings(&map),
        expected(&[
            &format!("ci .github/workflows remote_script:example.invalid {path}:6"),
            &format!("ci .github/workflows third_party_action:vendor/setup {path}:7"),
        ])
    );
}
