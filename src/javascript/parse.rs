use std::collections::{BTreeSet, HashMap};

use ast_grep_core::tree_sitter::LanguageExt;
use ast_grep_language::SupportLang;

use super::program::{FileKey, LoadedFile, Privilege, Program, Root, SiteSink};
use super::syntax::{self, FileIndex};
use crate::Error;
use crate::code::{MatchRequest, SyntaxNode};
use crate::deep::graph::Owner;

const SINK_NAME_LENGTH: usize = 60;

impl Program<'_> {
    pub(super) fn parse(&mut self, key: FileKey, text: &str) -> Result<LoadedFile, Error> {
        let (display, owner) = match &key.root {
            Root::Repository => (
                key.path.clone(),
                Owner::Repository {
                    scope: (self.input.scope_of)(&key.path),
                },
            ),
            Root::Package(index) => {
                let name = &self.input.packages.installed(*index).name;
                (
                    format!("{name}/{}", key.path),
                    Owner::Dependency {
                        package: name.clone(),
                    },
                )
            }
        };
        let document = SupportLang::Tsx.ast_grep(text);
        let root = document.root();
        let index = syntax::index(&root);
        let mut privileged: HashMap<String, Vec<Privilege>> = HashMap::new();
        let mut sites = Vec::new();
        for found in self.input.rules.matches(MatchRequest {
            path: &display,
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
        let specifiers: BTreeSet<String> = index
            .loads
            .iter()
            .map(|load| load.specifier.clone())
            .collect();
        let mut resolved = HashMap::new();
        for specifier in specifiers {
            if privileged.contains_key(&specifier) {
                continue;
            }
            if let Some(target) = self.resolve(&key, &specifier)? {
                resolved.insert(specifier, target);
            }
        }
        Ok(LoadedFile {
            key,
            display,
            owner,
            index,
            privileged,
            sites,
            resolved,
        })
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
