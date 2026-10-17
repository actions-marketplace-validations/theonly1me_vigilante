use crate::risks::{Change, review_of, risk_lines};

const PATH: &str = ".github/workflows/test.yml";

fn workflow(trigger: &str, step: &str) -> String {
    format!("on: {trigger}\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n{step}")
}

fn change_of(base: &str, head: &str) -> Vec<String> {
    risk_lines(&review_of(Change {
        base: &[(PATH, base)],
        head: &[(PATH, head)],
    }))
}

#[test]
fn a_pull_request_title_in_a_run_step_is_a_blocking_risk() {
    let lines = change_of(
        &workflow("pull_request", "      - run: echo hello\n"),
        &workflow(
            "pull_request",
            "      - run: echo \"${{ github.event.pull_request.title }}\"\n",
        ),
    );

    assert_eq!(
        lines,
        vec![format!(
            "{PATH} exec:test:github.event.pull_request.title:untrusted blocks=true approved=false"
        )]
    );
}

#[test]
fn an_event_number_in_a_run_step_is_not_a_risk() {
    let lines = change_of(
        &workflow("pull_request", "      - run: echo hello\n"),
        &workflow(
            "pull_request",
            "      - run: |\n          echo \"${{ github.event.pull_request.number }}\"\n",
        ),
    );

    assert_eq!(lines, Vec::<String>::new());
}

#[test]
fn a_commit_pin_that_becomes_a_branch_is_a_changed_risk() {
    let lines = change_of(
        &workflow(
            "push",
            "      - uses: vendor/action@0123456789abcdef0123456789abcdef01234567\n",
        ),
        &workflow("push", "      - uses: vendor/action@main\n"),
    );

    assert_eq!(
        lines,
        vec![
            ".github/workflows third_party_action:vendor/action:unpinned blocks=false approved=false"
        ]
    );
}

#[test]
fn a_new_tag_of_an_action_is_not_a_changed_risk() {
    let lines = change_of(
        &workflow("push", "      - uses: vendor/action@v1\n"),
        &workflow("push", "      - uses: vendor/action@v2\n"),
    );

    assert_eq!(lines, Vec::<String>::new());
}

#[test]
fn a_checkout_of_the_pull_request_head_under_pull_request_target_blocks() {
    let checkout = "      - uses: actions/checkout@v4\n        with:\n          ref: ${{ github.event.pull_request.head.sha }}\n";
    let lines = change_of(
        &workflow("pull_request_target", "      - uses: actions/checkout@v4\n"),
        &workflow("pull_request_target", checkout),
    );

    assert_eq!(
        lines,
        vec![format!(
            "{PATH} exec:test:actions/checkout:untrusted blocks=true approved=false"
        )]
    );
}

#[test]
fn a_checkout_of_the_pull_request_head_under_pull_request_is_not_a_risk() {
    let checkout = "      - uses: actions/checkout@v4\n        with:\n          ref: ${{ github.event.pull_request.head.sha }}\n";
    let lines = change_of(
        &workflow("pull_request", "      - uses: actions/checkout@v4\n"),
        &workflow("pull_request", checkout),
    );

    assert_eq!(lines, Vec::<String>::new());
}
