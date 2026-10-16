use super::{Found, allow_rule, claude, detection};
use crate::document::Node;
use crate::{Capability, Detection, Error, Power, SourceFile};

const FENCE: &str = "---";
const ALLOWED_TOOLS: &str = "allowed-tools";
const HOOKS: &str = "hooks";

pub fn is_frontmatter_file(path: &str) -> bool {
    let inside = |folder: &str| path.starts_with(folder) || path.contains(&format!("/{folder}"));
    (inside(".claude/skills/") && path.ends_with("/SKILL.md"))
        || (inside(".claude/commands/") && path.ends_with(".md"))
}

pub fn detect(file: SourceFile) -> Result<Vec<Detection>, Error> {
    let Some(yaml) = frontmatter(file.text) else {
        return Ok(Vec::new());
    };
    let document = Node::parse(&yaml)
        .map_err(|error| Error::new(format!("cannot parse {}: {error}", file.path)))?;
    let mut found = allowed_tools(&document);
    found.extend(claude::hooks(&document));
    Ok(found.into_iter().map(detection).collect())
}

fn frontmatter(text: &str) -> Option<String> {
    let mut lines = text.lines();
    if lines.next()?.trim_end() != FENCE {
        return None;
    }
    let mut yaml = vec![String::new()];
    let mut is_read = false;
    for line in lines {
        if line.trim_end() == FENCE {
            return Some(yaml.join("\n"));
        }
        let starts_key = !line.is_empty() && !line.starts_with([' ', '\t', '-', '#']);
        if starts_key {
            let key = line.split(':').next().unwrap_or_default().trim();
            is_read = key == ALLOWED_TOOLS || key == HOOKS;
        }
        yaml.push(if is_read {
            line.to_string()
        } else {
            String::new()
        });
    }
    None
}

fn allowed_tools(document: &Node) -> Vec<Found> {
    let Some(tools) = document.get(ALLOWED_TOOLS) else {
        return Vec::new();
    };
    let rules: Vec<(&str, usize)> = match tools.text() {
        Some(text) => rules_in(text)
            .into_iter()
            .map(|rule| (rule, tools.line))
            .collect(),
        None => tools
            .items()
            .iter()
            .filter_map(|item| Some((item.text()?, item.line)))
            .collect(),
    };
    rules
        .into_iter()
        .filter(|(rule, _)| allow_rule::grants_power(rule))
        .map(|(rule, line)| Found {
            power: Power::new(Capability::ToolAllow).with_detail(rule),
            line,
        })
        .collect()
}

fn rules_in(text: &str) -> Vec<&str> {
    let mut rules = Vec::new();
    let mut depth = 0_usize;
    let mut start = 0;
    for (index, character) in text.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' | ' ' | '\t' if depth == 0 => {
                rules.push(&text[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    rules.push(&text[start..]);
    rules.into_iter().filter(|rule| !rule.is_empty()).collect()
}
