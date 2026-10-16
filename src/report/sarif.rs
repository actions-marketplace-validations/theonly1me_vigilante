use std::collections::BTreeSet;

use serde_json::{Value, json};

use super::notes::call_path_text;
use super::risks::input_of;
use super::summary;
use crate::{Capability, Coverage, Finding, Review, RiskFinding};

const SCHEMA: &str = "https://json.schemastore.org/sarif-2.1.0.json";
const INFORMATION_URI: &str = "https://github.com/theonly1me/vigilante";

pub fn render(review: &Review) -> String {
    let capabilities: BTreeSet<Capability> = review
        .findings
        .iter()
        .chain(&review.tests)
        .map(|finding| finding.power.capability)
        .chain(review.risks.iter().map(|risk| risk.power.capability))
        .collect();
    let rules: Vec<Value> = capabilities
        .into_iter()
        .map(|capability| {
            json!({
                "id": capability.name(),
                "name": capability.name(),
                "shortDescription": { "text": summary(capability) },
            })
        })
        .collect();
    let finding_results = review.findings.iter().map(|finding| {
        result_of(Graded {
            finding,
            blocks: review.blocks(finding),
        })
    });
    let test_results = review.tests.iter().map(|finding| {
        result_of(Graded {
            finding,
            blocks: finding.denied,
        })
    });
    let risk_results = review.risks.iter().map(|risk| {
        risk_result_of(GradedRisk {
            risk,
            blocks: review.risk_blocks(risk),
        })
    });
    let results: Vec<Value> = finding_results
        .chain(test_results)
        .chain(risk_results)
        .collect();
    let gap_level = if review.coverage == Coverage::Block {
        "error"
    } else {
        "warning"
    };
    let notifications: Vec<Value> = review
        .skipped
        .iter()
        .map(|gap| {
            let level = if review.new_gaps.contains(gap) {
                gap_level
            } else {
                "note"
            };
            json!({
                "level": level,
                "message": { "text": format!("{}: {}", gap.source, gap.reason) },
                "locations": [{
                    "physicalLocation": { "artifactLocation": { "uri": gap.source } },
                }],
            })
        })
        .collect();
    let report = json!({
        "$schema": SCHEMA,
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "vigilante",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": INFORMATION_URI,
                    "rules": rules,
                },
            },
            "results": results,
            "invocations": [{
                "executionSuccessful": true,
                "toolExecutionNotifications": notifications,
            }],
        }],
    });
    serde_json::to_string_pretty(&report).unwrap_or_default() + "\n"
}

struct Graded<'a> {
    finding: &'a Finding,
    blocks: bool,
}

fn result_of(graded: Graded) -> Value {
    let finding = graded.finding;
    json!({
        "ruleId": finding.power.capability.name(),
        "level": if graded.blocks { "error" } else { "note" },
        "message": { "text": message_of(finding) },
        "locations": [{
            "physicalLocation": {
                "artifactLocation": { "uri": finding.evidence.path },
                "region": { "startLine": finding.evidence.line },
            },
        }],
    })
}

fn message_of(finding: &Finding) -> String {
    let message = format!(
        "{} in {} ({}): {}",
        summary(finding.power.capability),
        finding.scope.path,
        finding.scope.surface.name(),
        finding.power
    );
    match call_path_text(&finding.evidence) {
        Some(call_path) => format!("{message} via {call_path}"),
        None => message,
    }
}

struct GradedRisk<'a> {
    risk: &'a RiskFinding,
    blocks: bool,
}

fn risk_result_of(graded: GradedRisk) -> Value {
    let risk = graded.risk;
    let message = format!(
        "{} in {}, {}: {}",
        summary(risk.power.capability),
        risk.scope.path,
        input_of(risk.class),
        risk.identity
    );
    json!({
        "ruleId": risk.power.capability.name(),
        "level": if graded.blocks { "error" } else { "note" },
        "message": { "text": message },
        "locations": [{
            "physicalLocation": {
                "artifactLocation": { "uri": risk.evidence.path },
                "region": { "startLine": risk.evidence.line },
            },
        }],
    })
}
