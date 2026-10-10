mod expressions;

use crate::document::{Node, Value};
use crate::{Capability, Detection, Power};
pub use expressions::without_expression_braces;
use expressions::{expressions, secrets};

pub const WORKFLOWS: &str = ".github/workflows";
const UNTRUSTED_TRIGGERS: &[&str] = &["pull_request_target", "workflow_run"];
const FIRST_PARTY_OWNERS: &[&str] = &["actions/", "github/"];

pub fn is_workflow(path: &str) -> bool {
    path.strip_prefix(WORKFLOWS)
        .and_then(|rest| rest.strip_prefix('/'))
        .is_some_and(|name| {
            !name.contains('/') && (name.ends_with(".yml") || name.ends_with(".yaml"))
        })
}

pub struct Workflow<'a> {
    pub document: &'a Node,
    pub text: &'a str,
}

pub fn detect(source: Workflow) -> Vec<Detection> {
    let workflow = source.document;
    let mut detections = Vec::new();
    let jobs = workflow.get("jobs").map(Node::entries).unwrap_or_default();
    if let Some(triggers) = workflow.get("on") {
        detections.extend(untrusted_triggers(triggers));
    }
    let permission_blocks = workflow
        .get("permissions")
        .into_iter()
        .chain(jobs.iter().filter_map(|job| job.value.get("permissions")));
    detections.extend(permission_blocks.flat_map(token_permissions));
    let uses = jobs.iter().flat_map(|job| {
        let steps = job.value.get("steps").map(Node::items).unwrap_or_default();
        job.value
            .get("uses")
            .into_iter()
            .chain(steps.iter().filter_map(|step| step.get("uses")))
    });
    detections.extend(uses.filter_map(third_party_action));
    detections.extend(expressions(source.text).iter().flat_map(secrets));
    detections.extend(
        jobs.iter()
            .filter_map(|job| job.value.get("secrets"))
            .filter(|secrets| secrets.text() == Some("inherit"))
            .map(|secrets| {
                detection(Found {
                    power: Power::new(Capability::Secret).with_detail("inherit"),
                    line: secrets.line,
                })
            }),
    );
    detections
}

fn untrusted_triggers(triggers: &Node) -> Vec<Detection> {
    let names: Vec<(&str, usize)> = match &triggers.value {
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
    };
    names
        .into_iter()
        .filter(|(name, _)| UNTRUSTED_TRIGGERS.contains(name))
        .map(|(name, line)| {
            detection(Found {
                power: Power::new(Capability::UntrustedTrigger).with_detail(name),
                line,
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

fn third_party_action(uses: &Node) -> Option<Detection> {
    let reference = uses.text()?.trim();
    let is_first_party = reference.starts_with("./")
        || FIRST_PARTY_OWNERS
            .iter()
            .any(|owner| reference.starts_with(owner));
    if is_first_party {
        return None;
    }
    let action = reference.split('@').next().unwrap_or(reference);
    let power = Power::new(Capability::ThirdPartyAction).with_detail(action);
    Some(detection(Found {
        power,
        line: uses.line,
    }))
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
