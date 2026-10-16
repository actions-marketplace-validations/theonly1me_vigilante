use super::{Found, detection};
use crate::document::Located;
use crate::{Capability, Detection, Power};

const BUILT_IN_SECRET: &str = "GITHUB_TOKEN";
const ALL_SECRETS: &str = "all";
const EXPRESSION_START: &str = "${{";
const EXPRESSION_END: &str = "}}";

pub fn without_expression_braces(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find(EXPRESSION_START) {
        output.push_str(&rest[..start]);
        output.push_str("$((");
        rest = &rest[start + EXPRESSION_START.len()..];
        let Some(end) = rest.find(EXPRESSION_END) else {
            break;
        };
        output.push_str(&rest[..end]);
        output.push_str("))");
        rest = &rest[end + EXPRESSION_END.len()..];
    }
    output.push_str(rest);
    output
}

pub fn expressions(text: &str) -> Vec<Located<'_>> {
    let mut found = Vec::new();
    let mut offset = 0;
    while let Some(start) = text[offset..].find(EXPRESSION_START) {
        let body_start = offset + start + EXPRESSION_START.len();
        let Some(length) = text[body_start..].find(EXPRESSION_END) else {
            break;
        };
        found.push(Located {
            text: &text[body_start..body_start + length],
            line: text[..body_start].matches('\n').count() + 1,
        });
        offset = body_start + length + EXPRESSION_END.len();
    }
    found
}

pub fn secrets(located: &Located) -> Vec<Detection> {
    secret_names(located.text)
        .filter(|name| !name.eq_ignore_ascii_case(BUILT_IN_SECRET))
        .map(|name| {
            detection(Found {
                power: Power::new(Capability::Secret).with_detail(name),
                line: located.line,
            })
        })
        .collect()
}

fn secret_names(text: &str) -> impl Iterator<Item = &str> {
    const CONTEXT: &str = "secrets";
    text.match_indices(CONTEXT).filter_map(move |(index, _)| {
        let follows_name = text[..index]
            .chars()
            .next_back()
            .is_some_and(|character| is_name_character(character) || character == '.');
        let is_in_literal = text[..index].matches('\'').count() % 2 == 1;
        if follows_name || is_in_literal {
            return None;
        }
        let rest = &text[index + CONTEXT.len()..];
        if let Some(member) = rest.strip_prefix('.') {
            let name = &member[..name_length(member)];
            return (!name.is_empty()).then_some(name);
        }
        if let Some(index_text) = rest.strip_prefix('[') {
            return Some(quoted_name(index_text.trim_start()).unwrap_or(ALL_SECRETS));
        }
        let continues_name = rest.chars().next().is_some_and(is_name_character);
        (!continues_name).then_some(ALL_SECRETS)
    })
}

fn quoted_name(text: &str) -> Option<&str> {
    let quote = text
        .chars()
        .next()
        .filter(|character| "'\"".contains(*character))?;
    let body = &text[1..];
    let name = &body[..body.find(quote)?];
    (!name.is_empty() && name_length(name) == name.len()).then_some(name)
}

fn name_length(text: &str) -> usize {
    text.find(|character: char| !is_name_character(character))
        .unwrap_or(text.len())
}

fn is_name_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_' || character == '-'
}
