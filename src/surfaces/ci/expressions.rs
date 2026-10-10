use super::{Found, detection};
use crate::document::Located;
use crate::{Capability, Detection, Power};

const BUILT_IN_SECRET: &str = "GITHUB_TOKEN";
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
    const PREFIX: &str = "secrets.";
    text.match_indices(PREFIX).filter_map(move |(index, _)| {
        let follows_identifier = text[..index]
            .chars()
            .next_back()
            .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_');
        let rest = &text[index + PREFIX.len()..];
        let length = rest
            .find(|character: char| {
                !(character.is_ascii_alphanumeric() || character == '_' || character == '-')
            })
            .unwrap_or(rest.len());
        (!follows_identifier && length > 0).then_some(&rest[..length])
    })
}
