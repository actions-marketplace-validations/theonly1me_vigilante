use serde_json::{Value, json};

use crate::ScanReport;

pub fn scan_text(report: &ScanReport) -> String {
    let mut lines = Vec::new();
    for (scope, powers) in &report.map.scopes {
        for (power, evidence) in powers {
            let location = evidence
                .first()
                .map(|first| format!("{}:{}", first.path, first.line))
                .unwrap_or_default();
            lines.push(format!(
                "{:<6} {:<32} {:<40} {location}",
                scope.surface.name(),
                scope.path,
                power.to_string()
            ));
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
                .iter()
                .map(|(power, evidence)| json!({ "power": power, "evidence": evidence }))
                .collect();
            json!({ "surface": scope.surface, "scope": scope.path, "powers": powers })
        })
        .collect();
    let output = json!({ "powers": scopes, "not_analyzed": report.not_analyzed });
    serde_json::to_string_pretty(&output).unwrap_or_default() + "\n"
}
