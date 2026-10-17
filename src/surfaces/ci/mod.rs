mod expressions;
mod steps;

use super::Detected;
use crate::document::{Node, Value};
use crate::{Capability, Detection, Power};
pub use expressions::without_expression_braces;
use expressions::{expressions, secrets};
use steps::{StepInput, action_facts, step_facts};

pub const WORKFLOWS: &str = ".github/workflows";
const UNTRUSTED_TRIGGERS: &[&str] = &["pull_request_target", "workflow_run"];
const PRIVILEGED_TRIGGERS: &[&str] = &["pull_request_target", "workflow_run", "issue_comment"];
const ACTION_FILES: &[&str] = &["action.yml", "action.yaml"];
const COMPOSITE_JOB: &str = "runs";

pub fn is_workflow(path: &str) -> bool {
    path.strip_prefix(WORKFLOWS)
        .and_then(|rest| rest.strip_prefix('/'))
        .is_some_and(|name| {
            !name.contains('/') && (name.ends_with(".yml") || name.ends_with(".yaml"))
        })
}

pub fn is_composite_action(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    ACTION_FILES.contains(&name)
}

pub struct Workflow<'a> {
    pub document: &'a Node,
    pub text: &'a str,
}

pub fn detect(source: Workflow) -> Detected {
    let workflow = source.document;
    let mut detected = Detected::default();
    let jobs = workflow.get("jobs").map(Node::entries).unwrap_or_default();
    let triggers = workflow.get("on").map(trigger_names).unwrap_or_default();
    let privileged = triggers
        .iter()
        .any(|(name, _)| PRIVILEGED_TRIGGERS.contains(name));
    detected.detections.extend(untrusted_triggers(&triggers));
    let permission_blocks = workflow
        .get("permissions")
        .into_iter()
        .chain(jobs.iter().filter_map(|job| job.value.get("permissions")));
    detected
        .detections
        .extend(permission_blocks.flat_map(token_permissions));
    for job in jobs {
        if let Some(uses) = job.value.get("uses") {
            detected.extend(action_facts(uses));
        }
        for step in job.value.get("steps").map(Node::items).unwrap_or_default() {
            detected.extend(step_facts(StepInput {
                job: &job.key,
                step,
                privileged,
            }));
        }
        if let Some(secrets) = job.value.get("secrets")
            && secrets.text() == Some("inherit")
        {
            detected.detections.push(detection(Found {
                power: Power::new(Capability::Secret).with_detail("inherit"),
                line: secrets.line,
            }));
        }
    }
    detected
        .detections
        .extend(expressions(source.text).iter().flat_map(secrets));
    detected
}

pub fn detect_composite(source: Workflow) -> Detected {
    let mut detected = Detected::default();
    let steps = source
        .document
        .get("runs")
        .and_then(|runs| runs.get("steps"))
        .map(Node::items)
        .unwrap_or_default();
    for step in steps {
        detected.extend(step_facts(StepInput {
            job: COMPOSITE_JOB,
            step,
            privileged: false,
        }));
    }
    detected
}

fn trigger_names(triggers: &Node) -> Vec<(&str, usize)> {
    match &triggers.value {
        Value::Text(name) => vec![(name.as_str(), triggers.line)],
        Value::List(items) => items
            .iter()
            .filter_map(|item| Some((item.text()?, item.line)))
            .collect(),
        Value::Map(entries) => entries
            .iter()
            .map(|entry| (entry.key.as_str(), entry.line))
            .collect(),
        Value::Empty => Vec::new(),
    }
}

fn untrusted_triggers(triggers: &[(&str, usize)]) -> Vec<Detection> {
    triggers
        .iter()
        .filter(|(name, _)| UNTRUSTED_TRIGGERS.contains(name))
        .map(|(name, line)| {
            detection(Found {
                power: Power::new(Capability::UntrustedTrigger).with_detail(*name),
                line: *line,
            })
        })
        .collect()
}

fn token_permissions(permissions: &Node) -> Vec<Detection> {
    if permissions.text() == Some("write-all") {
        let power = Power::new(Capability::TokenWrite).with_detail("all");
        return vec![detection(Found {
            power,
            line: permissions.line,
        })];
    }
    permissions
        .entries()
        .iter()
        .filter(|entry| entry.value.text() == Some("write"))
        .map(|entry| {
            let power = match entry.key.as_str() {
                "id-token" => Power::new(Capability::Oidc),
                scope => Power::new(Capability::TokenWrite).with_detail(scope),
            };
            detection(Found {
                power,
                line: entry.line,
            })
        })
        .collect()
}

struct Found {
    power: Power,
    line: usize,
}

fn detection(found: Found) -> Detection {
    Detection {
        power: found.power,
        line: found.line,
        rule: "github-workflow".to_string(),
    }
}
