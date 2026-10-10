use std::collections::BTreeSet;

use super::summary;
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
        (ReviewMode::Check, 0) if review.stale.is_empty() => format!("{} is current", Lock::FILE),
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
        code(&format!(
            "{}:{}",
            finding.evidence.path, finding.evidence.line
        )),
    )
}

fn result(review: &Review) -> String {
    if review.passed() {
        return "**Result: pass.**".to_string();
    }
    let mut reasons = Vec::new();
    let unlocked = review
        .findings
        .iter()
        .filter(|finding| review.blocks(finding) && !finding.denied)
        .count();
    if unlocked > 0 {
        reasons.push(format!(
            "{unlocked} {} not in {}. Run `vigilante lock` and commit the file.",
            plural(Plural {
                count: unlocked,
                one: "power is",
                many: "powers are"
            }),
            Lock::FILE
        ));
    }
    let denied = review
        .findings
        .iter()
        .filter(|finding| finding.denied)
        .count();
    if denied > 0 {
        reasons.push(format!(
            "{denied} {} denied by the policy in vigilante.toml.",
            plural(Plural {
                count: denied,
                one: "power is",
                many: "powers are"
            })
        ));
    }
    if !review.stale.is_empty() {
        let stale = review.stale.len();
        reasons.push(format!(
            "{stale} {} stale. Run `vigilante lock` to remove {}.",
            plural(Plural {
                count: stale,
                one: "lock entry is",
                many: "lock entries are"
            }),
            plural(Plural {
                count: stale,
                one: "it",
                many: "them"
            })
        ));
    }
    format!("**Result: fail.** {}", reasons.join(" "))
}

struct Plural {
    count: usize,
    one: &'static str,
    many: &'static str,
}

fn plural(word: Plural) -> &'static str {
    if word.count == 1 { word.one } else { word.many }
}

fn code(text: &str) -> String {
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
