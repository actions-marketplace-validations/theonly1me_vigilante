mod branch;
mod codeowners;
mod owners;
mod protected;

use std::path::PathBuf;

use branch::branch_rules;
use owners::owner_items;

const GUIDE: &str =
    "https://github.com/theonly1me/vigilante/blob/main/docs/github-action.md#protect-the-check";

pub struct DoctorRequest {
    pub repository: PathBuf,
    pub check: String,
}

pub struct Doctor {
    items: Vec<Item>,
}

struct Item {
    passed: bool,
    text: String,
}

pub fn doctor(request: DoctorRequest) -> Doctor {
    let mut items = owner_items(&request);
    items.extend(branch_items(&request));
    Doctor { items }
}

impl Doctor {
    pub fn passed(&self) -> bool {
        self.items.iter().all(|item| item.passed)
    }

    pub fn render(&self) -> String {
        let mut lines = vec!["## Vigilante doctor".to_string(), String::new()];
        lines.extend(self.items.iter().map(|item| {
            let mark = if item.passed { "x" } else { " " };
            format!("- [{mark}] {}", item.text)
        }));
        lines.push(String::new());
        let problems = self.items.iter().filter(|item| !item.passed).count();
        lines.push(match problems {
            0 => "**Result: pass.**".to_string(),
            count => format!(
                "**Result: fail.** {count} {}. Without these rules, a pull request can approve its own powers. See {GUIDE}.",
                if count == 1 { "problem" } else { "problems" }
            ),
        });
        lines.join("\n") + "\n"
    }
}

fn branch_items(request: &DoctorRequest) -> Vec<Item> {
    let rules = match branch_rules(&request.repository) {
        Ok(rules) => rules,
        Err(reason) => {
            return vec![Item {
                passed: false,
                text: format!("Vigilante cannot read the branch rules: {reason}."),
            }];
        }
    };
    let branch = &rules.branch;
    let wanted = request.check.to_lowercase();
    let has_check = rules
        .required_checks
        .iter()
        .any(|check| check.to_lowercase().contains(&wanted));
    let checks = if rules.required_checks.is_empty() {
        "The branch requires no checks.".to_string()
    } else {
        format!(
            "The required checks are `{}`.",
            rules.required_checks.join("`, `")
        )
    };
    vec![
        Item {
            passed: rules.requires_pull_request,
            text: if rules.requires_pull_request {
                format!("`{branch}` requires a pull request.")
            } else {
                format!("`{branch}` does not require a pull request.")
            },
        },
        Item {
            passed: rules.requires_code_owner_review,
            text: if rules.requires_code_owner_review {
                format!("`{branch}` requires a review from code owners.")
            } else {
                format!("`{branch}` does not require a review from code owners.")
            },
        },
        Item {
            passed: has_check,
            text: if has_check {
                format!(
                    "`{branch}` requires a check with `{}` in its name.",
                    request.check
                )
            } else {
                format!(
                    "`{branch}` does not require a check with `{}` in its name. {checks}",
                    request.check
                )
            },
        },
    ]
}
