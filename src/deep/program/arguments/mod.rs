mod sites;
mod syntax;

use std::collections::HashMap;

use super::{FileIndex, Language};
use crate::code::SyntaxNode;
pub use sites::CallFacts;
use sites::CallSite;
use syntax::{
    assignments, callee_text, contains_source, identifiers, input_arguments, is_call, is_literal,
    parameter_names,
};

const SHELLS: &[&str] = &[
    "sh",
    "bash",
    "/bin/sh",
    "/bin/bash",
    "zsh",
    "cmd",
    "cmd.exe",
    "powershell",
    "pwsh",
];
const SHELL_FLAGS: &[&str] = &["-c", "/c", "-Command"];
const SHELL_OPTIONS: &[&str] = &["shell=True", "shell: true", "shell:true"];
const EXPANSION_DEPTH: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ArgumentFacts {
    pub constant: bool,
    pub shell: bool,
    pub untrusted: bool,
    pub from_parameters: bool,
}

impl ArgumentFacts {
    fn merged(self, other: Self) -> Self {
        Self {
            constant: self.constant && other.constant,
            shell: self.shell || other.shell,
            untrusted: self.untrusted || other.untrusted,
            from_parameters: self.from_parameters || other.from_parameters,
        }
    }
}

pub struct FactsInput<'a, 'r> {
    pub root: &'a SyntaxNode<'r>,
    pub language: &'a dyn Language,
    pub index: &'a FileIndex,
}

struct FunctionFacts {
    parameters: Vec<String>,
    parameters_are_sources: bool,
    assignments: Vec<(String, String, usize)>,
}

pub fn call_facts(input: FactsInput) -> CallFacts {
    let FactsInput {
        root,
        language,
        index,
    } = input;
    let sources = language.untrusted_sources();
    let mut functions: HashMap<usize, FunctionFacts> = HashMap::new();
    let mut facts = CallFacts::default();
    for call in root.dfs().filter(is_call) {
        let Some(arguments) = call.field("arguments") else {
            continue;
        };
        let start = call.range().start;
        let definition = index.innermost(start);
        let function = functions.entry(definition).or_insert_with(|| {
            function_facts(FunctionQuery {
                root,
                index,
                definition,
                sources,
            })
        });
        let text = arguments.text().to_string();
        let expanded = expand(Expansion {
            text: &text,
            assignments: &function.assignments,
            before: start,
        });
        let inputs = input_arguments(&arguments);
        let input_text = inputs
            .iter()
            .map(|input| input.text().to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let expanded_input = expand(Expansion {
            text: &input_text,
            assignments: &function.assignments,
            before: start,
        });
        let from_parameters = identifiers(&expanded_input)
            .any(|word| function.parameters.iter().any(|name| name == word));
        let callee = callee_text(&call);
        let method = callee.rsplit(['.', ':']).next().unwrap_or_default();
        let found = ArgumentFacts {
            constant: inputs.iter().all(is_literal),
            shell: SHELL_OPTIONS.iter().any(|option| expanded.contains(option))
                || language.shell_callees().contains(&method)
                || runs_a_shell(&text),
            untrusted: contains_source(&expanded_input, sources)
                || (function.parameters_are_sources && from_parameters),
            from_parameters,
        };
        facts.push(CallSite {
            range: call.range(),
            arguments: arguments.range(),
            facts: found,
        });
    }
    facts
}

struct FunctionQuery<'a, 'r> {
    root: &'a SyntaxNode<'r>,
    index: &'a FileIndex,
    definition: usize,
    sources: &'a [&'a str],
}

fn function_facts(query: FunctionQuery) -> FunctionFacts {
    let range = &query.index.definitions[query.definition].range;
    let node = if query.definition == 0 {
        Some(query.root.clone())
    } else {
        query.root.dfs().find(|node| node.range() == *range)
    };
    let Some(node) = node else {
        return FunctionFacts {
            parameters: Vec::new(),
            parameters_are_sources: false,
            assignments: Vec::new(),
        };
    };
    let parameters_text = node
        .field("parameters")
        .map(|parameters| parameters.text().to_string())
        .unwrap_or_default();
    FunctionFacts {
        parameters: parameter_names(&node),
        parameters_are_sources: contains_source(&parameters_text, query.sources),
        assignments: assignments(&node),
    }
}

struct Expansion<'a> {
    text: &'a str,
    assignments: &'a [(String, String, usize)],
    before: usize,
}

fn expand(expansion: Expansion) -> String {
    let mut expanded = expansion.text.to_string();
    let mut frontier = expansion.text.to_string();
    for _ in 0..EXPANSION_DEPTH {
        let mut next = String::new();
        for word in identifiers(&frontier) {
            if let Some((_, value, _)) = expansion
                .assignments
                .iter()
                .rev()
                .find(|(name, _, start)| name == word && *start < expansion.before)
            {
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

fn runs_a_shell(text: &str) -> bool {
    let quoted =
        |word: &str| text.contains(&format!("\"{word}\"")) || text.contains(&format!("'{word}'"));
    SHELLS.iter().any(|shell| quoted(shell)) && SHELL_FLAGS.iter().any(|flag| quoted(flag))
}
