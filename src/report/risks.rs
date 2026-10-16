use super::markdown::code;
use super::notes::evidence_text;
use crate::deep::RiskClass;
use crate::{Lock, Review, ReviewMode, RiskFinding};

const MAXIMUM_RISK_LINES: usize = 10;

pub fn risk_section(review: &Review) -> Vec<String> {
    if review.risks.is_empty() {
        return Vec::new();
    }
    let (heading, scope) = match review.mode {
        ReviewMode::Diff { .. } => ("### Changed risks", "These calls are new or changed."),
        ReviewMode::Check => ("### Risks", "These calls need approval."),
    };
    let mut lines = vec![
        heading.to_string(),
        String::new(),
        format!(
            "{scope} A call that gets untrusted input, or a shell command that is not constant, blocks until {} approves it.",
            Lock::FILE
        ),
        String::new(),
    ];
    let mut ordered: Vec<&RiskFinding> = review.risks.iter().collect();
    ordered.sort_by_key(|risk| !review.risk_blocks(risk));
    lines.extend(ordered.into_iter().take(MAXIMUM_RISK_LINES).map(|risk| {
        let status = if review.risk_blocks(risk) {
            " **not approved**"
        } else if risk.blocks {
            " (approved)"
        } else {
            ""
        };
        format!(
            "- {} {} {}{status}: {}",
            code(&risk.scope.path),
            code(&risk.power.to_string()),
            input_of(risk.class),
            code(&evidence_text(&risk.evidence))
        )
    }));
    let more = review.risks.len().saturating_sub(MAXIMUM_RISK_LINES);
    if more > 0 {
        lines.push(format!("- {more} more risks are in the JSON report."));
    }
    lines.push(String::new());
    lines
}

pub fn input_of(class: RiskClass) -> &'static str {
    match class {
        RiskClass::Constant => "with constant input",
        RiskClass::Dynamic => "with input that changes at run time",
        RiskClass::Shell => "through a shell, with a command that is not constant",
        RiskClass::Untrusted => "with untrusted input",
        RiskClass::Unpinned => "with a reference that can change",
    }
}
