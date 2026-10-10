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
