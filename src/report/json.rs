use serde::Serialize;

use crate::Review;

#[derive(Serialize)]
struct Report<'a> {
    passed: bool,
    #[serde(flatten)]
    review: &'a Review,
}

pub fn render(review: &Review) -> String {
    let report = Report {
        passed: review.passed(),
        review,
    };
    serde_json::to_string_pretty(&report).unwrap_or_default() + "\n"
}
