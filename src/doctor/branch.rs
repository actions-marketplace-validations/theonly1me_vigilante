use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::process::{Finished, LimitedRun, run_limited};

const TIME_LIMIT: Duration = Duration::from_secs(60);

#[derive(Default)]
pub struct BranchRules {
    pub branch: String,
    pub requires_pull_request: bool,
    pub requires_code_owner_review: bool,
    pub required_checks: Vec<String>,
}

pub fn branch_rules(repository: &Path) -> Result<BranchRules, String> {
    let gh = Gh { repository };
    let project = gh.json(&["repo", "view", "--json", "nameWithOwner,defaultBranchRef"])?;
    let name = text_at(&project, "/nameWithOwner")?;
    let branch = text_at(&project, "/defaultBranchRef/name")?;
    let mut rules = BranchRules {
        branch: branch.to_string(),
        ..BranchRules::default()
    };
    let rule_list = gh.json(&["api", &format!("repos/{name}/rules/branches/{branch}")])?;
    for rule in rule_list.as_array().into_iter().flatten() {
        let parameters = &rule["parameters"];
        match rule["type"].as_str() {
            Some("pull_request") => {
                rules.requires_pull_request = true;
                rules.requires_code_owner_review |= parameters["require_code_owner_review"] == true;
            }
            Some("required_status_checks") => rules
                .required_checks
                .extend(contexts(&parameters["required_status_checks"])),
            _ => {}
        }
    }
    let protection_path = format!("repos/{name}/branches/{branch}/protection");
    if let Ok(protection) = gh.json(&["api", &protection_path]) {
        let reviews = &protection["required_pull_request_reviews"];
        rules.requires_pull_request |= reviews.is_object();
        rules.requires_code_owner_review |= reviews["require_code_owner_reviews"] == true;
        let checks = &protection["required_status_checks"];
        rules.required_checks.extend(
            checks["contexts"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_string),
        );
        rules.required_checks.extend(contexts(&checks["checks"]));
    }
    rules.required_checks.sort();
    rules.required_checks.dedup();
    Ok(rules)
}

fn contexts(checks: &Value) -> Vec<String> {
    checks
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|check| check["context"].as_str())
        .map(str::to_string)
        .collect()
}

fn text_at<'a>(value: &'a Value, pointer: &str) -> Result<&'a str, String> {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("`gh` gave no `{pointer}`"))
}

struct Gh<'a> {
    repository: &'a Path,
}

impl Gh<'_> {
    fn json(&self, arguments: &[&str]) -> Result<Value, String> {
        let mut command = Command::new("gh");
        command.args(arguments).current_dir(self.repository);
        let finished = run_limited(LimitedRun {
            command: &mut command,
            input: Vec::new(),
            deadline: Instant::now() + TIME_LIMIT,
        })
        .map_err(|error| match error.kind() {
            std::io::ErrorKind::NotFound => "`gh` is not installed".to_string(),
            _ => format!("cannot run `gh`: {error}"),
        })?;
        let output = match finished {
            Finished::Exited(output) => output,
            Finished::OutOfTime => return Err("`gh` did not finish in 60 seconds".to_string()),
        };
        if !output.status.success() {
            let message = String::from_utf8_lossy(&output.stderr);
            return Err(message.lines().next().unwrap_or("`gh` failed").to_string());
        }
        serde_json::from_slice(&output.stdout)
            .map_err(|error| format!("`gh` gave invalid JSON: {error}"))
    }
}
