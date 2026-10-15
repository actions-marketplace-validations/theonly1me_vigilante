use serde_json::{Value, json};

use crate::PowerMap;

pub fn power_map_text(map: &PowerMap) -> String {
    let mut lines = Vec::new();
    for (scope, powers) in &map.scopes {
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
        return "No powers found.\n".to_string();
    }
    lines.join("\n") + "\n"
}

pub fn power_map_json(map: &PowerMap) -> String {
    let scopes: Vec<Value> = map
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
    serde_json::to_string_pretty(&scopes).unwrap_or_default() + "\n"
}
