use std::collections::HashMap;
use std::num::NonZero;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use ast_grep_core::tree_sitter::LanguageExt;
use ast_grep_language::SupportLang;

use super::program::{Privilege, SiteSink};
use super::syntax::{self, FileIndex};
use crate::Error;
use crate::code::{MatchRequest, Rules, SyntaxNode};

const SINK_NAME_LENGTH: usize = 60;

pub struct Source {
    pub display: String,
    pub text: String,
}

pub struct Parsed {
    pub index: FileIndex,
    pub privileged: HashMap<String, Vec<Privilege>>,
    pub sites: Vec<SiteSink>,
}

pub fn parse_all(rules: &Rules, sources: &[Source]) -> Result<Vec<Parsed>, Error> {
    let next = AtomicUsize::new(0);
    let workers = thread::available_parallelism()
        .map_or(1, NonZero::get)
        .min(sources.len())
        .max(1);
    let mut finished: Vec<(usize, Parsed)> = thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                scope.spawn(|| {
                    let mut done = Vec::new();
                    loop {
                        let position = next.fetch_add(1, Ordering::Relaxed);
                        let Some(source) = sources.get(position) else {
                            return done;
                        };
                        done.push((position, parse(rules, source)));
                    }
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .map_err(|_| Error::new("a JavaScript parser thread stopped"))
            })
            .collect::<Result<Vec<_>, Error>>()
            .map(|parts| parts.into_iter().flatten().collect())
    })?;
    finished.sort_by_key(|(position, _)| *position);
    Ok(finished.into_iter().map(|(_, parsed)| parsed).collect())
}

fn parse(rules: &Rules, source: &Source) -> Parsed {
    let document = SupportLang::Tsx.ast_grep(&source.text);
    let root = document.root();
    let index = syntax::index(&root);
    let mut privileged: HashMap<String, Vec<Privilege>> = HashMap::new();
    let mut sites = Vec::new();
    for found in rules.matches(MatchRequest {
        path: &source.display,
        language: SupportLang::Tsx,
        root: &root,
    }) {
        let privilege = Privilege {
            capability: found.capability,
            rule: found.rule,
        };
        if found.node.kind() == "string" {
            let specifier = found.node.text().trim_matches(['"', '\'', '`']).to_string();
            privileged.entry(specifier).or_default().push(privilege);
        } else {
            sites.push(SiteSink {
                definition: innermost(&index, found.node.range().start),
                line: syntax::line_of(&found.node),
                name: sink_name(&found.node),
                privilege,
            });
        }
    }
    Parsed {
        index,
        privileged,
        sites,
    }
}

fn innermost(index: &FileIndex, offset: usize) -> usize {
    index
        .definitions
        .iter()
        .enumerate()
        .filter(|(_, definition)| definition.range.contains(&offset))
        .min_by_key(|(_, definition)| definition.range.len())
        .map_or(0, |(position, _)| position)
}

fn sink_name(node: &SyntaxNode) -> String {
    let callee = match node.kind().as_ref() {
        "call_expression" => node.field("function"),
        "new_expression" => node.field("constructor"),
        "assignment_expression" => node.field("left"),
        _ => None,
    };
    let text = callee.map_or_else(
        || node.text().to_string(),
        |callee| callee.text().to_string(),
    );
    let first_line = text.lines().next().unwrap_or_default();
    first_line
        .split('(')
        .next()
        .unwrap_or_default()
        .chars()
        .take(SINK_NAME_LENGTH)
        .collect()
}
