use std::fs;
use std::path::Path;

use globset::{GlobBuilder, GlobMatcher};

const LOCATIONS: &[&str] = &[".github/CODEOWNERS", "CODEOWNERS", "docs/CODEOWNERS"];

pub struct CodeOwners {
    pub path: &'static str,
    rules: Vec<Rule>,
}

struct Rule {
    matchers: Vec<GlobMatcher>,
    owners: Vec<String>,
}

impl CodeOwners {
    pub fn find(repository: &Path) -> Option<Self> {
        LOCATIONS.iter().find_map(|path| {
            let text = fs::read_to_string(repository.join(path)).ok()?;
            Some(Self {
                path,
                rules: text.lines().filter_map(rule_of).collect(),
            })
        })
    }

    pub fn owners_of(&self, path: &str) -> &[String] {
        self.rules
            .iter()
            .rev()
            .find(|rule| rule.matchers.iter().any(|matcher| matcher.is_match(path)))
            .map_or(&[], |rule| rule.owners.as_slice())
    }
}

fn rule_of(line: &str) -> Option<Rule> {
    let content = line.split_once(" #").map_or(line, |(content, _)| content);
    let mut words = content.split_whitespace();
    let pattern = words.next().filter(|pattern| !pattern.starts_with('#'))?;
    let matchers = globs_of(pattern)
        .iter()
        .map(|glob| {
            GlobBuilder::new(glob)
                .literal_separator(true)
                .build()
                .map(|glob| glob.compile_matcher())
        })
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    Some(Rule {
        matchers,
        owners: words.map(str::to_string).collect(),
    })
}

fn globs_of(pattern: &str) -> Vec<String> {
    let core = pattern.trim_start_matches('/').trim_end_matches('/');
    let is_anchored = pattern.starts_with('/') || core.contains('/');
    let base = if is_anchored {
        core.to_string()
    } else {
        format!("**/{core}")
    };
    let last_part_has_wildcard = core
        .rsplit('/')
        .next()
        .is_some_and(|part| part.contains('*'));
    if pattern.ends_with('/') {
        vec![format!("{base}/**")]
    } else if last_part_has_wildcard {
        vec![base]
    } else {
        vec![format!("{base}/**"), base]
    }
}
