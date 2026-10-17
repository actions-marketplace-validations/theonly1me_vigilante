use super::{Found, detection};
use crate::deep::RiskClass;
use crate::document::Node;
use crate::surfaces::download::downloaded_script_host;
use crate::surfaces::{Detected, SurfaceRisk};
use crate::{Capability, Power};

const FIRST_PARTY_OWNERS: &[&str] = &["actions/", "github/"];
const UNTRUSTED_FIELDS: &[&str] = &[
    ".title",
    ".body",
    ".message",
    ".author.name",
    ".author.email",
    ".head.ref",
    ".head.label",
    ".head.repo.default_branch",
    ".page_name",
    ".head_branch",
];
const HEAD_REFERENCES: &[&str] = &[
    "github.event.pull_request.head.",
    "github.head_ref",
    "github.event.workflow_run.head_",
    "refs/pull/",
];
const EXPRESSION_START: &str = "$((";
const EXPRESSION_END: &str = "))";
const COMMIT_LENGTH: usize = 40;

pub struct StepInput<'a> {
    pub job: &'a str,
    pub step: &'a Node,
    pub privileged: bool,
}

pub fn step_facts(input: StepInput) -> Detected {
    let StepInput {
        job,
        step,
        privileged,
    } = input;
    let mut detected = Detected::default();
    if let Some(run) = step.get("run") {
        detected.extend(script_facts(Script { job, node: run }));
    }
    let Some(uses) = step.get("uses") else {
        return detected;
    };
    detected.extend(action_facts(uses));
    let action = uses.text().unwrap_or_default();
    let with = step.get("with");
    if action.starts_with("actions/github-script@")
        && let Some(script) = with.and_then(|with| with.get("script"))
    {
        detected.extend(script_facts(Script { job, node: script }));
    }
    let checks_out_the_head = with
        .and_then(|with| with.get("ref"))
        .and_then(Node::text)
        .is_some_and(|reference| HEAD_REFERENCES.iter().any(|head| reference.contains(head)));
    if privileged && action.starts_with("actions/checkout@") && checks_out_the_head {
        detected.risks.push(untrusted_exec(UntrustedExec {
            job,
            sink: "actions/checkout",
            line: uses.line,
        }));
    }
    detected
}

pub fn action_facts(uses: &Node) -> Detected {
    let mut detected = Detected::default();
    let Some(reference) = uses.text().map(str::trim) else {
        return detected;
    };
    let is_first_party = reference.starts_with("./")
        || FIRST_PARTY_OWNERS
            .iter()
            .any(|owner| reference.starts_with(owner));
    if is_first_party {
        return detected;
    }
    let (action, pin) = reference.split_once('@').unwrap_or((reference, ""));
    let power = Power::new(Capability::ThirdPartyAction).with_detail(action);
    let is_commit =
        pin.len() == COMMIT_LENGTH && pin.chars().all(|digit| digit.is_ascii_hexdigit());
    if !is_commit {
        detected.risks.push(SurfaceRisk {
            power: power.clone(),
            class: RiskClass::Unpinned,
            identity: format!("{power}:{}", RiskClass::Unpinned.name()),
            line: uses.line,
            call_path: vec![reference.to_string()],
        });
    }
    detected.detections.push(detection(Found {
        power,
        line: uses.line,
    }));
    detected
}

struct Script<'a> {
    job: &'a str,
    node: &'a Node,
}

fn script_facts(script: Script) -> Detected {
    let mut detected = Detected::default();
    let Some(text) = script.node.text() else {
        return detected;
    };
    let first_line = script.node.line + usize::from(text.contains('\n'));
    for (index, line) in logical_lines(text) {
        let line_number = first_line + index;
        for field in untrusted_fields(&line) {
            detected.risks.push(untrusted_exec(UntrustedExec {
                job: script.job,
                sink: field,
                line: line_number,
            }));
        }
        if let Some(host) = downloaded_script_host(&line) {
            detected.detections.push(detection(Found {
                power: Power::new(Capability::RemoteScript).with_detail(host),
                line: line_number,
            }));
        }
    }
    detected
}

fn logical_lines(text: &str) -> Vec<(usize, String)> {
    let mut lines: Vec<(usize, String)> = Vec::new();
    let mut continues = false;
    for (index, line) in text.lines().enumerate() {
        let content = line.strip_suffix('\\').unwrap_or(line);
        match lines.last_mut() {
            Some((_, joined)) if continues => joined.push_str(content),
            _ => lines.push((index, content.to_string())),
        }
        continues = line.ends_with('\\');
    }
    lines
}

fn untrusted_fields(line: &str) -> Vec<&str> {
    let mut fields = Vec::new();
    let mut rest = line;
    while let Some(start) = rest.find(EXPRESSION_START) {
        let body = &rest[start + EXPRESSION_START.len()..];
        let end = body.find(EXPRESSION_END).unwrap_or(body.len());
        fields.extend(
            body[..end]
                .split(|character: char| {
                    !(character.is_ascii_alphanumeric() || "._-*[]".contains(character))
                })
                .filter(|token| is_untrusted(token)),
        );
        rest = &body[end..];
    }
    fields
}

fn is_untrusted(token: &str) -> bool {
    token == "github.head_ref"
        || (token.starts_with("github.event.")
            && UNTRUSTED_FIELDS.iter().any(|field| token.ends_with(field)))
}

struct UntrustedExec<'a> {
    job: &'a str,
    sink: &'a str,
    line: usize,
}

fn untrusted_exec(site: UntrustedExec) -> SurfaceRisk {
    let class = RiskClass::Untrusted;
    SurfaceRisk {
        power: Power::new(Capability::Exec),
        class,
        identity: format!("exec:{}:{}:{}", site.job, site.sink, class.name()),
        line: site.line,
        call_path: vec![site.job.to_string(), site.sink.to_string()],
    }
}
