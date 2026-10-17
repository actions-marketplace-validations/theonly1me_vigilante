use ast_grep_core::tree_sitter::LanguageExt;

use super::{FileIndex, Language, Privilege, PrivilegedQuery, SiteSink};
use crate::code::{MatchRequest, RuleMatch, Rules};

pub struct CanonicalInput<'a, 'r> {
    pub language: &'a dyn Language,
    pub rules: &'a Rules,
    pub display: &'a str,
    pub root: &'a crate::code::SyntaxNode<'r>,
    pub index: &'a FileIndex,
}

pub struct CanonicalSink {
    pub privilege: Privilege,
    pub specifiers: Vec<String>,
    pub site: SiteSink,
}

pub fn canonical_sinks(input: CanonicalInput) -> Vec<CanonicalSink> {
    let CanonicalInput {
        language,
        rules,
        display,
        root,
        index,
    } = input;
    let mut sinks = Vec::new();
    for call in language.canonical_calls(root) {
        let document = language.grammar().ast_grep(&call.snippet);
        let snippet_root = document.root();
        for found in rules.matches(MatchRequest {
            path: display,
            language: language.grammar(),
            root: &snippet_root,
        }) {
            if index.shadows(&found.node) {
                continue;
            }
            sinks.push(sink_of(SinkParts {
                language,
                rules,
                display,
                found: &found,
                index,
                original: &call.node,
            }));
        }
    }
    sinks
}

struct SinkParts<'a, 'r, 's> {
    language: &'a dyn Language,
    rules: &'a Rules,
    display: &'a str,
    found: &'a RuleMatch<'s>,
    index: &'a FileIndex,
    original: &'a crate::code::SyntaxNode<'r>,
}

fn sink_of(parts: SinkParts) -> CanonicalSink {
    let SinkParts {
        language,
        rules,
        display,
        found,
        index,
        original,
    } = parts;
    let is_privileged = |text: &str| {
        let snippet = language.grammar().ast_grep(text);
        rules
            .matches(MatchRequest {
                path: display,
                language: language.grammar(),
                root: &snippet.root(),
            })
            .iter()
            .any(|other| other.rule == found.rule)
    };
    let specifiers = language.privileged_modules(PrivilegedQuery {
        node: &found.node,
        is_privileged: &is_privileged,
    });
    let privilege = Privilege {
        capability: found.capability,
        rule: found.rule.clone(),
        inert: found.inert.clone(),
    };
    CanonicalSink {
        site: SiteSink {
            definition: index.innermost(original.range().start),
            line: original.start_pos().line() + 1,
            start: original.range().start,
            name: super::parse::sink_name(original),
            privilege: privilege.clone(),
        },
        privilege,
        specifiers,
    }
}
