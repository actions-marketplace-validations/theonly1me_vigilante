use std::collections::HashMap;
use std::ops::Range;

pub const INIT: &str = "init";

#[derive(Default)]
pub struct FileIndex {
    pub definitions: Vec<Definition>,
    pub bindings: HashMap<String, Binding>,
    pub exports: Vec<Export>,
    pub loads: Vec<Load>,
    pub ignored: Vec<Range<usize>>,
    pub supertypes: Vec<Supertypes>,
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
}
