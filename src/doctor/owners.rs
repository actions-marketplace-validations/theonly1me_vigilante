use std::iter::once;

use super::codeowners::CodeOwners;
use super::protected::{Protected, ProtectedPaths, protected_paths};
use super::{DoctorRequest, Item};

pub fn owner_items(request: &DoctorRequest) -> Vec<Item> {
    let Some(code_owners) = CodeOwners::find(&request.repository) else {
        return vec![Item {
            passed: false,
            text:
                "The repository has no CODEOWNERS file in `.github/`, the top folder, or `docs/`."
                    .to_string(),
        }];
    };
    let ProtectedPaths {
        mut entries,
        problem,
    } = protected_paths(&request.repository);
    entries.push(Protected {
        label: code_owners.path.to_string(),
        sample: code_owners.path.to_string(),
        files: Vec::new(),
    });
    let mut items: Vec<Item> = entries
        .iter()
        .map(|entry| {
            owner_item(&OwnerQuery {
                entry,
                code_owners: &code_owners,
            })
        })
        .collect();
    items.extend(problem.map(|problem| Item {
        passed: false,
        text: format!(
            "Vigilante cannot read the rule folders: {}.",
            problem.split_whitespace().collect::<Vec<_>>().join(" ")
        ),
    }));
    items
}

struct OwnerQuery<'a> {
    entry: &'a Protected,
    code_owners: &'a CodeOwners,
}

fn owner_item(query: &OwnerQuery) -> Item {
    let OwnerQuery { entry, code_owners } = query;
    if code_owners.owners_of(&entry.sample).is_empty() {
        return Item {
            passed: false,
            text: format!(
                "`{}` has no code owner in `{}`.",
                entry.label, code_owners.path
            ),
        };
    }
    let unowned: Vec<&str> = entry
        .files
        .iter()
        .map(String::as_str)
        .filter(|file| code_owners.owners_of(file).is_empty())
        .collect();
    if !unowned.is_empty() {
        return Item {
            passed: false,
            text: format!(
                "`{}` has no code owner for `{}` in `{}`.",
                entry.label,
                unowned.join("`, `"),
                code_owners.path
            ),
        };
    }
    let mut owners: Vec<&str> = Vec::new();
    for path in once(&entry.sample).chain(&entry.files) {
        for owner in code_owners.owners_of(path) {
            if !owners.contains(&owner.as_str()) {
                owners.push(owner);
            }
        }
    }
    Item {
        passed: true,
        text: format!(
            "`{}` has the code owners `{}`.",
            entry.label,
            owners.join("`, `")
        ),
    }
}
