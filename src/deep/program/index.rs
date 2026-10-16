use super::ArgumentFacts;
use crate::code::SyntaxNode;
use std::collections::HashMap;
use std::ops::Range;

pub const INIT: &str = "init";

#[derive(Default)]
pub struct FileIndex {
    pub definitions: Vec<Definition>,
    pub bindings: HashMap<String, Binding>,
    pub exports: Vec<Export>,
    pub loads: Vec<Load>,
    pub unresolved_loads: Vec<usize>,
    pub call_facts: HashMap<usize, ArgumentFacts>,
    pub ignored: Vec<Range<usize>>,
    pub supertypes: Vec<Supertypes>,
    pub members: Vec<Range<usize>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Supertypes {
    pub class: String,
    pub names: Vec<String>,
}

pub struct Definition {
    pub name: String,
    pub simple: String,
    pub range: Range<usize>,
    pub uses: Vec<Use>,
}

pub struct Use {
    pub line: usize,
    pub target: UseTarget,
}

pub enum UseTarget {
    Name(String),
    Member {
        object: String,
        property: String,
        call: bool,
    },
    Method(String),
    Load(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Load {
    pub specifier: String,
    pub line: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    pub specifier: String,
    pub imported: Imported,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Imported {
    Module,
    Named(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Export {
    Local {
        exported: String,
        local: String,
    },
    Forward {
        exported: Option<String>,
        specifier: String,
        imported: Option<String>,
    },
}

impl FileIndex {
    pub fn innermost(&self, offset: usize) -> usize {
        self.definitions
            .iter()
            .enumerate()
            .filter(|(_, definition)| definition.range.contains(&offset))
            .min_by_key(|(_, definition)| definition.range.len())
            .map_or(0, |(position, _)| position)
    }

    pub fn shadows(&self, node: &SyntaxNode) -> bool {
        let callee = match node.kind().as_ref() {
            "call_expression" | "call" => node.field("function"),
            "new_expression" => node.field("constructor"),
            _ => None,
        };
        let Some(callee) = callee.filter(|callee| callee.kind() == "identifier") else {
            return false;
        };
        let name = callee.text();
        let own = &self.definitions[1..];
        own.iter().any(|definition| {
            definition.simple == name
                && !self.members.contains(&definition.range)
                && !own.iter().any(|outer| {
                    outer.range != definition.range
                        && outer.range.start <= definition.range.start
                        && definition.range.end <= outer.range.end
                })
        })
    }
}
