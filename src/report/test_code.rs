use super::markdown::code;
use super::notes::evidence_text;
use crate::Review;

const MAXIMUM_TEST_LINES: usize = 10;

pub fn test_code_section(review: &Review) -> Vec<String> {
    if review.tests.is_empty() {
        return Vec::new();
    }
    let mut lines = vec![
        "### Test code".to_string(),
        String::new(),
        "Tests and fixtures run in CI and in agent sessions. These powers do not go in the lock, and they block only when a `deny` rule matches.".to_string(),
        String::new(),
    ];
    lines.extend(review.tests.iter().take(MAXIMUM_TEST_LINES).map(|finding| {
        let status = if finding.denied { " **denied**" } else { "" };
        format!(
            "- {} {}{status}: {}",
            code(&finding.scope.path),
            code(&finding.power.to_string()),
            code(&evidence_text(&finding.evidence))
        )
    }));
    let more = review.tests.len().saturating_sub(MAXIMUM_TEST_LINES);
    if more > 0 {
        lines.push(format!("- {more} more lines are in the JSON report."));
    }
    lines.push(String::new());
    lines
}
