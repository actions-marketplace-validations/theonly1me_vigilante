use std::collections::HashMap;

use super::syntax::identifiers;

const EXPANSION_DEPTH: usize = 3;

#[derive(Default)]
pub struct Assignments {
    by_name: HashMap<String, Vec<(usize, String)>>,
}

impl Assignments {
    pub fn new(list: Vec<(String, String, usize)>) -> Self {
        let mut by_name: HashMap<String, Vec<(usize, String)>> = HashMap::new();
        for (name, value, start) in list {
            by_name.entry(name).or_default().push((start, value));
        }
        Self { by_name }
    }

    fn latest_before(&self, name: &str, before: usize) -> Option<&str> {
        let starts = self.by_name.get(name)?;
        let earlier = starts.partition_point(|(start, _)| *start < before);
        starts[..earlier].last().map(|(_, value)| value.as_str())
    }
}

pub struct Expansion<'a> {
    pub text: &'a str,
    pub assignments: &'a Assignments,
    pub before: usize,
}

pub fn expand(expansion: Expansion) -> String {
    let mut expanded = expansion.text.to_string();
    let mut frontier = expansion.text.to_string();
    for _ in 0..EXPANSION_DEPTH {
        let mut next = String::new();
        for word in identifiers(&frontier) {
            if let Some(value) = expansion.assignments.latest_before(word, expansion.before) {
                next.push(' ');
                next.push_str(value);
            }
        }
        if next.is_empty() {
            break;
        }
        expanded.push_str(&next);
        frontier = next;
    }
    expanded
}
