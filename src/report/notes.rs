use std::collections::BTreeSet;

use super::markdown::code;
use crate::{Coverage, Evidence, Note, Review, ReviewMode};

const CALL_SEPARATOR: &str = " → ";
const MAXIMUM_SKIPPED_LINES: usize = 10;

pub fn evidence_text(evidence: &Evidence) -> String {
    let location = format!("{}:{}", evidence.path, evidence.line);
    match call_path_text(evidence) {
        Some(call_path) => format!("{location} via {call_path}"),
        None => location,
    }
}

pub fn call_path_text(evidence: &Evidence) -> Option<String> {
    if evidence.call_path.is_empty() {
        return None;
    }
    let names: Vec<String> = evidence
        .call_path
        .iter()
        .map(|function| short_name(function))
        .collect();
    Some(names.join(CALL_SEPARATOR))
}

pub fn markdown_section(review: &Review) -> Vec<String> {
    if review.notes.is_empty() {
        return Vec::new();
    }
    let mut rows = BTreeSet::new();
    let shown: Vec<&Note> = review
        .notes
        .iter()
        .filter(|note| !note.hidden)
        .filter(|note| rows.insert((&note.scope, &note.power, &note.through)))
        .collect();
    let mut lines = vec![
        "### New packages on call paths".to_string(),
        String::new(),
        "These packages are new on a call path to a power. They need no change to vigilante.lock. Check each new dependency.".to_string(),
        String::new(),
    ];
    if !shown.is_empty() {
        lines.push("| Scope | Power | Through | Evidence |".to_string());
        lines.push("| --- | --- | --- | --- |".to_string());
        lines.extend(shown.iter().map(|note| {
            format!(
                "| {} ({}) | {} | {} | {} |",
                code(&note.scope.path),
                note.scope.surface.name(),
                code(&note.power.to_string()),
                code(&note.through),
                code(&evidence_text(&note.evidence))
            )
        }));
        lines.push(String::new());
    }
    let hidden: BTreeSet<&str> = review
        .notes
        .iter()
        .filter(|note| note.hidden)
        .map(|note| note.power.capability.name())
        .collect();
    if !hidden.is_empty() {
        let names: Vec<&str> = hidden.into_iter().collect();
        lines.push(format!(
            "Hidden notes for low-signal powers: {} ({}). They are in the JSON report.",
            review.notes.iter().filter(|note| note.hidden).count(),
            names.join(", ")
        ));
        lines.push(String::new());
    }
    lines
}

pub fn skipped_section(review: &Review) -> Vec<String> {
    if review.skipped.is_empty() {
        return Vec::new();
    }
    let mut lines = vec![
        "### Not analyzed".to_string(),
        String::new(),
        "Vigilante did not read this code, so the report does not show its powers.".to_string(),
        String::new(),
    ];
    if review.coverage == Coverage::Block && !review.new_gaps.is_empty() {
        lines.push(
            "The lines marked new fail the check, because `[policy] coverage` is `\"block\"`."
                .to_string(),
        );
        lines.push(String::new());
    }
    lines.extend(
        review
            .skipped
            .iter()
            .take(MAXIMUM_SKIPPED_LINES)
            .map(|skip| {
                let is_diff = matches!(review.mode, ReviewMode::Diff { .. });
                let marker = if is_diff && review.new_gaps.contains(skip) {
                    " (new)"
                } else {
                    ""
                };
                format!("- {}{marker}: {}", code(&skip.source), skip.reason)
            }),
    );
    let more = review.skipped.len().saturating_sub(MAXIMUM_SKIPPED_LINES);
    if more > 0 {
        lines.push(format!("- {more} more lines are in the JSON report."));
    }
    lines.push(String::new());
    lines
}

fn short_name(function: &str) -> String {
    let Some(start) = function.find(|character| character != '(' && character != '*') else {
        return function.to_string();
    };
    let (receiver_opening, rest) = function.split_at(start);
    let search_end = rest.find('[').unwrap_or(rest.len());
    let name_start = rest[..search_end].rfind('/').map_or(0, |index| index + 1);
    format!("{receiver_opening}{}", &rest[name_start..])
}
