use std::collections::HashMap;
use std::num::NonZero;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use ast_grep_core::tree_sitter::LanguageExt;

use super::{FileIndex, Language, Privilege, SiteSink};
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

pub struct ParseInput<'a> {
    pub language: &'a dyn Language,
    pub rules: &'a Rules,
    pub sources: &'a [Source],
}

pub fn parse_all(input: ParseInput) -> Result<Vec<Parsed>, Error> {
    let ParseInput {
        language,
        rules,
        sources,
    } = input;
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
                        done.push((
                            position,
                            parse(SourceRules {
                                language,
                                source,
                                rules,
                            }),
                        ));
                    }
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .map_err(|_| Error::new("a parser thread stopped"))
            })
            .collect::<Result<Vec<_>, Error>>()
            .map(|parts| parts.into_iter().flatten().collect())
    })?;
    finished.sort_by_key(|(position, _)| *position);
    Ok(finished.into_iter().map(|(_, parsed)| parsed).collect())
}

struct SourceRules<'a> {
    language: &'a dyn Language,
    source: &'a Source,
    rules: &'a Rules,
}

fn parse(input: SourceRules) -> Parsed {
    let SourceRules {
        language,
        source,
        rules,
    } = input;
    let document = language.grammar().ast_grep(&source.text);
    let root = document.root();
    let index = language.index(&root);
    let mut privileged: HashMap<String, Vec<Privilege>> = HashMap::new();
    let mut sites = Vec::new();
    for found in rules.matches(MatchRequest {
        path: &source.display,
        language: language.grammar(),
        root: &root,
    }) {
        let start = found.node.range().start;
        if index.ignored.iter().any(|range| range.contains(&start)) {
            continue;
        }
        let privilege = Privilege {
            capability: found.capability,
            rule: found.rule,
        };
        if let Some(specifier) = language.privileged_module(&found.node) {
            privileged.entry(specifier).or_default().push(privilege);
        } else {
            sites.push(SiteSink {
                definition: index.innermost(found.node.range().start),
                line: found.node.start_pos().line() + 1,
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

fn sink_name(node: &SyntaxNode) -> String {
    let callee = match node.kind().as_ref() {
        "call_expression" | "call" => node.field("function"),
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
