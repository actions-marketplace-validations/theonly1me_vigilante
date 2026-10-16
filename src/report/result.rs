use crate::{Coverage, Lock, Review};

pub fn result(review: &Review) -> String {
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
    let stale = review.stale.len() + review.stale_risks.len();
    if stale > 0 {
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
    let risks = review
        .risks
        .iter()
        .filter(|risk| review.risk_blocks(risk))
        .count();
    if risks > 0 {
        reasons.push(format!(
            "{risks} {} not approved in {}. Review {}, then run `vigilante lock` to approve {}.",
            plural(Plural {
                count: risks,
                one: "risk is",
                many: "risks are"
            }),
            Lock::FILE,
            plural(Plural {
                count: risks,
                one: "it",
                many: "them"
            }),
            plural(Plural {
                count: risks,
                one: "it",
                many: "them"
            })
        ));
    }
    let gaps = review.new_gaps.len();
    if review.coverage == Coverage::Block && gaps > 0 {
        reasons.push(format!(
            "{gaps} new {} not analyzed.",
            plural(Plural {
                count: gaps,
                one: "source is",
                many: "sources are"
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
