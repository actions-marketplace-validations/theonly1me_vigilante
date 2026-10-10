use crate::document::{Located, Node, Value};
use crate::{Capability, Detection, Power};

const WORKFLOW_FOLDER: &str = ".github/workflows/";
const UNTRUSTED_TRIGGERS: &[&str] = &["pull_request_target", "workflow_run"];
const FIRST_PARTY_OWNERS: &[&str] = &["actions/", "github/"];
const BUILT_IN_SECRET: &str = "GITHUB_TOKEN";
const EXPRESSION_START: &str = "${{";
const EXPRESSION_END: &str = "}}";

pub fn is_workflow(path: &str) -> bool {
    path.strip_prefix(WORKFLOW_FOLDER).is_some_and(|name| {
        !name.contains('/') && (name.ends_with(".yml") || name.ends_with(".yaml"))
    })
}

pub fn without_expression_braces(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find(EXPRESSION_START) {
        output.push_str(&rest[..start]);
        output.push_str("$((");
        rest = &rest[start + EXPRESSION_START.len()..];
        let Some(end) = rest.find(EXPRESSION_END) else {
            break;
        };
        output.push_str(&rest[..end]);
        output.push_str("))");
        rest = &rest[end + EXPRESSION_END.len()..];
    }
    output.push_str(rest);
    output
}

pub fn detect(workflow: &Node) -> Vec<Detection> {
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
    detections.extend(workflow.texts().iter().flat_map(secrets));
    detections.extend(
        jobs.iter()
            .filter_map(|job| job.value.get("secrets"))
            .filter(|secrets| secrets.text() == Some("inherit"))
            .map(|secrets| {
                detection(
                    Power::new(Capability::Secret).with_detail("inherit"),
                    secrets.line,
                )
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
            detection(
                Power::new(Capability::UntrustedTrigger).with_detail(name),
                line,
            )
        })
        .collect()
}

fn token_permissions(permissions: &Node) -> Vec<Detection> {
    if permissions.text() == Some("write-all") {
        let power = Power::new(Capability::TokenWrite).with_detail("all");
        return vec![detection(power, permissions.line)];
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
            detection(power, entry.line)
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
    Some(detection(power, uses.line))
}

fn secrets(located: &Located) -> Vec<Detection> {
    secret_names(located.text)
        .filter(|name| !name.eq_ignore_ascii_case(BUILT_IN_SECRET))
        .map(|name| {
            detection(
                Power::new(Capability::Secret).with_detail(name),
                located.line,
            )
        })
        .collect()
}

fn secret_names(text: &str) -> impl Iterator<Item = &str> {
    const PREFIX: &str = "secrets.";
    text.match_indices(PREFIX).filter_map(move |(index, _)| {
        let follows_identifier = text[..index]
            .chars()
            .next_back()
            .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_');
        let rest = &text[index + PREFIX.len()..];
        let length = rest
            .find(|character: char| {
                !(character.is_ascii_alphanumeric() || character == '_' || character == '-')
            })
            .unwrap_or(rest.len());
        (!follows_identifier && length > 0).then_some(&rest[..length])
    })
}

fn detection(power: Power, line: usize) -> Detection {
    Detection {
        power,
        line,
        rule: "github-workflow".to_string(),
    }
}
