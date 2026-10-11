mod bindings;
mod fallback;
mod targets;
mod typed;

use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::typed::TypedCalls;
use super::{FileKey, Language, LoadedFile, Program, Resolver, Root, UseTarget};
use crate::Power;
use crate::deep::ScopeRoots;
use crate::deep::graph::{Call, Function, FunctionId, Graph, Owner, Sink, Target};
use typed::TypedUse;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Unit {
    Scope(String),
    Package(usize),
}

#[derive(Clone, Copy)]
struct InFile<'a> {
    file: usize,
    name: &'a str,
}

#[derive(Clone, Copy)]
struct FileUse<'a> {
    file: usize,
    target: &'a UseTarget,
}

pub struct LinkInput<'p, 'a> {
    pub program: &'p Program<'a>,
    pub roots: &'p BTreeSet<String>,
    pub types: Option<&'p TypedCalls>,
}

pub struct Linked {
    pub graph: Graph,
    pub scopes: Vec<ScopeRoots>,
}

struct Linker<'p> {
    files: &'p [LoadedFile],
    ids: &'p HashMap<FileKey, usize>,
    language: &'p dyn Language,
    resolver: &'p dyn Resolver,
    types: Option<&'p TypedCalls>,
    graph: Graph,
    functions: Vec<Vec<FunctionId>>,
    by_name: HashMap<Unit, HashMap<String, Vec<FunctionId>>>,
    by_full_name: HashMap<Unit, HashMap<String, Vec<FunctionId>>>,
    in_unit: HashMap<Unit, Vec<FunctionId>>,
    fallbacks: HashMap<usize, FunctionId>,
}

pub fn link(input: LinkInput) -> Linked {
    let LinkInput {
        program,
        roots,
        types,
    } = input;
    let mut linker = Linker {
        files: &program.files,
        ids: &program.ids,
        language: program.language(),
        resolver: program.resolver(),
        types,
        graph: Graph::default(),
        functions: Vec::new(),
        by_name: HashMap::new(),
        by_full_name: HashMap::new(),
        in_unit: HashMap::new(),
        fallbacks: HashMap::new(),
    };
    linker.add_functions();
    for file in 0..linker.files.len() {
        linker.link_file(file);
    }
    let mut scopes: BTreeMap<String, Vec<FunctionId>> = BTreeMap::new();
    for (position, file) in linker.files.iter().enumerate() {
        if let (Root::Repository, Owner::Repository { scope }) = (&file.key.root, &file.owner)
            && roots.contains(&file.key.path)
        {
            scopes
                .entry(scope.clone())
                .or_default()
                .extend(&linker.functions[position]);
        }
    }
    Linked {
        graph: linker.graph,
        scopes: scopes
            .into_iter()
            .map(|(scope, roots)| ScopeRoots { scope, roots })
            .collect(),
    }
}

impl Linker<'_> {
    fn add_functions(&mut self) {
        for file in self.files {
            let unit = unit_of(file);
            let mut ids = Vec::new();
            for (position, definition) in file.index.definitions.iter().enumerate() {
                let id = self.graph.add_function(Function {
                    name: format!("{}.{}", self.language.stem(&file.display), definition.name),
                    owner: file.owner.clone(),
                    path: file.display.clone(),
                });
                if position > 0 {
                    self.by_name
                        .entry(unit.clone())
                        .or_default()
                        .entry(definition.simple.clone())
                        .or_default()
                        .push(id);
                    self.by_full_name
                        .entry(unit.clone())
                        .or_default()
                        .entry(definition.name.clone())
                        .or_default()
                        .push(id);
                }
                self.in_unit.entry(unit.clone()).or_default().push(id);
                ids.push(id);
            }
            self.functions.push(ids);
        }
    }

    fn link_file(&mut self, file: usize) {
        let loaded = &self.files[file];
        for (definition, item) in loaded.index.definitions.iter().enumerate() {
            let from = self.functions[file][definition];
            for item_use in &item.uses {
                for target in self.call_targets(TypedUse {
                    item: FileUse {
                        file,
                        target: &item_use.target,
                    },
                    line: item_use.line,
                }) {
                    self.graph.add_call(Call {
                        from,
                        line: item_use.line,
                        target,
                    });
                }
            }
        }
        for site in &loaded.sites {
            let sink = self.graph.add_sink(Sink {
                name: site.name.clone(),
                power: Power::new(site.privilege.capability),
                rule: site.privilege.rule.clone(),
            });
            let from = self.functions[file][site.definition];
            self.graph.add_call(Call {
                from,
                line: site.line,
                target: Target::Sink(sink),
            });
        }
        if !self.language.imports_run_code() {
            return;
        }
        let init = self.functions[file][0];
        for load in &loaded.index.loads {
            let Some(target) = loaded
                .resolved
                .get(&load.specifier)
                .and_then(|key| self.ids.get(key))
            else {
                continue;
            };
            let target_init = self.functions[*target][0];
            self.graph.add_call(Call {
                from: init,
                line: load.line,
                target: Target::Function(target_init),
            });
        }
    }
}

fn unit_of(file: &LoadedFile) -> Unit {
    match (&file.key.root, &file.owner) {
        (Root::Package(index), _) => Unit::Package(*index),
        (Root::Repository, Owner::Repository { scope }) => Unit::Scope(scope.clone()),
        (Root::Repository, Owner::Dependency { package }) => Unit::Scope(package.clone()),
    }
}

fn specifier_module(specifier: &str) -> String {
    specifier.trim_start_matches("node:").to_string()
}
