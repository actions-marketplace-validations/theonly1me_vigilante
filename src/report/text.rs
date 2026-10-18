use serde_json::{Value, json};

use super::examples::{Candidates, listed, shown};
use crate::{Power, ScanReport, Scope};

struct Lookup<'a> {
    report: &'a ScanReport,
    scope: &'a Scope,
    power: &'a Power,
}

fn candidates_of(lookup: Lookup) -> Candidates {
    let Lookup {
        report,
        scope,
        power,
    } = lookup;
    Candidates {
        found: &report.map.scopes[scope][power],
        extra: report
            .examples
            .scopes
            .get(scope)
            .and_then(|powers| powers.get(power)),
    }
}

pub fn scan_text(report: &ScanReport) -> String {
    let mut lines = Vec::new();
    for (scope, powers) in &report.map.scopes {
        for power in powers.keys() {
            let examples = shown(candidates_of(Lookup {
                report,
                scope,
                power,
            }));
            for example in examples {
                lines.push(format!(
                    "{:<6} {:<32} {:<40} {}:{}",
                    scope.surface.name(),
                    scope.path,
                    power.to_string(),
                    example.path,
                    example.line
                ));
            }
        }
    }
    if lines.is_empty() {
        lines.push("No powers found.".to_string());
    }
    lines.extend(
        report
            .not_analyzed
            .iter()
            .map(|gap| format!("not analyzed: {}: {}", gap.source, gap.reason)),
    );
    lines.join("\n") + "\n"
}

pub fn scan_json(report: &ScanReport) -> String {
    let scopes: Vec<Value> = report
        .map
        .scopes
        .iter()
        .map(|(scope, powers)| {
            let powers: Vec<Value> = powers
                .keys()
                .map(|power| {
                    let evidence = listed(candidates_of(Lookup {
                        report,
                        scope,
                        power,
                    }));
                    json!({ "power": power, "evidence": evidence })
                })
                .collect();
            json!({ "surface": scope.surface, "scope": scope.path, "powers": powers })
        })
        .collect();
    let output = json!({ "powers": scopes, "not_analyzed": report.not_analyzed });
    serde_json::to_string_pretty(&output).unwrap_or_default() + "\n"
}
