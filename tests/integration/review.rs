use vigilante::{Lock, ScanRequest, Source};

use crate::support::{Branches, TestRepository, finding_lines};

const MAIN_AND_FEATURE: Branches = Branches {
    base: "main",
    head: "feature",
};

const MCP_SERVER: &str =
    r#"{"mcpServers": {"fs-tools": {"command": "npx", "args": ["-y", "@acme/fs-mcp"]}}}"#;

fn start_feature(repository: &TestRepository) {
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
}

fn write_current_lock(repository: &TestRepository) {
    let map = vigilante::scan(ScanRequest {
        repository: repository.directory.path().to_path_buf(),
        source: Source::WorkingDirectory,
    })
    .expect("scan");
    repository.write(&[(Lock::FILE, &Lock::of(&map).render())]);
}

#[test]
fn diff_fails_until_the_lock_lists_the_new_powers() {
    let repository = TestRepository::new();
    repository.write(&[("README.md", "# app\n")]);
    repository.commit("base");
    start_feature(&repository);
    repository.write(&[
        (".mcp.json", MCP_SERVER),
        (
            ".github/workflows/release.yml",
            "on: push\npermissions:\n  contents: write\njobs: {}\n",
        ),
        (
            "services/billing/export.py",
            "import subprocess; subprocess.run(['true'])\n",
        ),
    ]);
    repository.commit("add powers");

    let before_lock = repository.diff(MAIN_AND_FEATURE);
    write_current_lock(&repository);
    repository.commit("lock powers");
    let after_lock = repository.diff(MAIN_AND_FEATURE);

    assert_eq!(
        finding_lines(&before_lock),
        [
            "code services exec locked=false denied=false",
            "agent .mcp.json mcp_server:fs-tools: npx -y @acme/fs-mcp locked=false denied=false",
            "ci .github/workflows/release.yml token_write:contents locked=false denied=false",
        ]
    );
    assert!(!before_lock.passed());
    assert_eq!(
        finding_lines(&after_lock),
        [
            "code services exec locked=true denied=false",
            "agent .mcp.json mcp_server:fs-tools: npx -y @acme/fs-mcp locked=true denied=false",
            "ci .github/workflows/release.yml token_write:contents locked=true denied=false",
        ]
    );
    assert!(after_lock.passed());
}

#[test]
fn diff_compares_with_the_merge_base_not_the_moving_main_branch() {
    let repository = TestRepository::new();
    repository.write(&[
        (
            "services/worker.py",
            "import subprocess; subprocess.run(['true'])\n",
        ),
        ("services/app.py", "value = 1\n"),
    ]);
    repository.commit("base");
    start_feature(&repository);
    repository.write(&[("services/app.py", "value = 2\n")]);
    repository.commit("feature change");
    repository.git(&["checkout", "--quiet", "main"]);
    repository.remove("services/worker.py");
    repository.commit("main drops the worker");

    let review = repository.diff(MAIN_AND_FEATURE);

    assert_eq!(finding_lines(&review), Vec::<String>::new());
    assert!(review.passed());
}

#[test]
fn diff_reports_a_power_only_the_first_time_it_appears_in_a_scope() {
    let repository = TestRepository::new();
    repository.write(&[
        (
            "services/client.py",
            "import requests; requests.get('https://example.com')\n",
        ),
        ("web/app.py", "value = 1\n"),
    ]);
    repository.commit("base");
    start_feature(&repository);
    repository.write(&[
        ("services/other_client.py", "import httpx\n"),
        (
            "web/client.py",
            "import requests; requests.get('https://example.com')\n",
        ),
    ]);
    repository.commit("more network");

    let review = repository.diff(MAIN_AND_FEATURE);

    assert_eq!(
        finding_lines(&review),
        ["code web network locked=false denied=false"]
    );
}

#[test]
fn diff_reads_the_config_from_the_base_so_a_pull_request_cannot_hide_itself() {
    let repository = TestRepository::new();
    repository.write(&[("README.md", "# app\n")]);
    repository.commit("base");
    start_feature(&repository);
    repository.write(&[
        (
            "vigilante.toml",
            "[scan]\nexclude = [\"**\"]\n\n[policy]\nmode = \"report\"\n",
        ),
        (
            "tools/run.py",
            "import subprocess; subprocess.run(['true'])\n",
        ),
    ]);
    repository.commit("hide the power");

    let review = repository.diff(MAIN_AND_FEATURE);

    assert_eq!(
        finding_lines(&review),
        ["code tools exec locked=false denied=false"]
    );
    assert!(!review.passed());
}

#[test]
fn diff_does_not_report_an_action_that_another_workflow_already_uses() {
    let repository = TestRepository::new();
    let steps = "on: push\njobs:\n  ship:\n    steps:\n      - uses: docker/login-action@v3\n";
    repository.write(&[(".github/workflows/ci.yml", steps)]);
    repository.commit("base");
    start_feature(&repository);
    repository.write(&[(
        ".github/workflows/deploy.yml",
        &format!("{steps}      - uses: webfactory/ssh-agent@v0.9.0\n"),
    )]);
    repository.commit("add a deploy workflow");

    let review = repository.diff(MAIN_AND_FEATURE);

    assert_eq!(
        finding_lines(&review),
        ["ci .github/workflows third_party_action:webfactory/ssh-agent locked=false denied=false"]
    );
}
