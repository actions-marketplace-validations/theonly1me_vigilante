use std::collections::BTreeSet;

use serde_json::{Value, json};

use super::notes::call_path_text;
use super::summary;
use crate::{Capability, Finding, Review};

const SCHEMA: &str = "https://json.schemastore.org/sarif-2.1.0.json";
const INFORMATION_URI: &str = "https://github.com/theonly1me/vigilante";

pub fn render(review: &Review) -> String {
    let capabilities: BTreeSet<Capability> = review
        .findings
        .iter()
        .map(|finding| finding.power.capability)
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
    let results: Vec<Value> = review
        .findings
        .iter()
        .map(|finding| {
            let level = if review.blocks(finding) {
                "error"
            } else {
                "note"
            };
            json!({
                "ruleId": finding.power.capability.name(),
                "level": level,
                "message": {
                    "text": message_of(finding),
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": finding.evidence.path },
                        "region": { "startLine": finding.evidence.line },
                    },
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
        }],
    });
    serde_json::to_string_pretty(&report).unwrap_or_default() + "\n"
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
