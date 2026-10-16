use std::collections::BTreeSet;

use super::notes::{evidence_text, markdown_section, skipped_section};
use super::result::result;
use super::risks::risk_section;
use super::summary;
use super::test_code::test_code_section;
use crate::{Finding, Lock, Review, ReviewMode};

const MAXIMUM_CELL_LENGTH: usize = 120;

pub fn render(review: &Review) -> String {
    let shown: Vec<&Finding> = review.shown_findings().collect();
    let mut lines = vec![format!("## Vigilante: {}", title(review)), String::new()];
    if !shown.is_empty() {
        lines.push("| Severity | Scope | Power | What it means | Evidence | Status |".to_string());
        lines.push("| --- | --- | --- | --- | --- | --- |".to_string());
        lines.extend(shown.iter().map(|finding| row(Row { review, finding })));
        lines.push(String::new());
    }
    let hidden: BTreeSet<&str> = review
        .findings
        .iter()
        .filter(|finding| finding.hidden && !review.blocks(finding))
        .map(|finding| finding.power.capability.name())
        .collect();
    if !hidden.is_empty() {
        let names: Vec<&str> = hidden.into_iter().collect();
        lines.push(format!(
            "Hidden low-signal findings: {} ({}). They stay in {}.",
            review.hidden_count(),
            names.join(", "),
            Lock::FILE
        ));
        lines.push(String::new());
    }
    lines.extend(risk_section(review));
    lines.extend(markdown_section(review));
    lines.extend(test_code_section(review));
    lines.extend(skipped_section(review));
    if !review.stale.is_empty() {
        lines.push(format!("Stale entries in {}:", Lock::FILE));
        lines.extend(review.stale.iter().map(|entry| {
            format!(
                "- {} ({}): {}",
                code(&entry.scope.path),
                entry.scope.surface.name(),
                code(&entry.power.to_string())
            )
        }));
        lines.push(String::new());
    }
    lines.push(result(review));
    lines.join("\n") + "\n"
}

fn title(review: &Review) -> String {
    match (&review.mode, review.findings.len()) {
        (ReviewMode::Diff { .. }, 0) => "this pull request adds no new powers".to_string(),
        (ReviewMode::Diff { .. }, 1) => "this pull request adds 1 new power".to_string(),
        (ReviewMode::Diff { .. }, count) => format!("this pull request adds {count} new powers"),
        (ReviewMode::Check, 0) if review.passed() => format!("{} is current", Lock::FILE),
        (ReviewMode::Check, _) => format!("{} needs an update", Lock::FILE),
    }
}

struct Row<'a> {
    review: &'a Review,
    finding: &'a Finding,
}

fn row(row: Row) -> String {
    let finding = row.finding;
    let status = if finding.denied {
        "denied by policy"
    } else if finding.locked {
        "in lock"
    } else {
        "not in lock"
    };
    let marker = if row.review.blocks(finding) { "**" } else { "" };
    format!(
        "| {} | {} ({}) | {} | {} | {} | {marker}{status}{marker} |",
        finding.severity.name().to_uppercase(),
        code(&finding.scope.path),
        finding.scope.surface.name(),
        code(&finding.power.to_string()),
        summary(finding.power.capability),
        code(&evidence_text(&finding.evidence)),
    )
}

pub(super) fn code(text: &str) -> String {
    let single_line: String = text
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect();
    let mut shortened: String = single_line.chars().take(MAXIMUM_CELL_LENGTH).collect();
    if single_line.chars().count() > MAXIMUM_CELL_LENGTH {
        shortened.push_str("...");
    }
    let escaped = shortened.replace('|', "\\|");
    let longest_backtick_run = escaped
        .split(|character| character != '`')
        .map(str::len)
        .max()
        .unwrap_or(0);
    let fence = "`".repeat(longest_backtick_run + 1);
    if longest_backtick_run == 0 {
        format!("{fence}{escaped}{fence}")
    } else {
        format!("{fence} {escaped} {fence}")
    }
}
