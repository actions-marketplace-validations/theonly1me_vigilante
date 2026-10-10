mod bindings;
mod targets;

use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::paths::stem;
use super::program::{LoadedFile, Program, Root};
use crate::Power;
use crate::deep::ScopeRoots;
use crate::deep::graph::{Edge, Function, FunctionId, Graph, Owner, Sink, Target};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Unit {
    Scope(String),
    Package(usize),
}

pub struct Linked {
    pub graph: Graph,
    pub scopes: Vec<ScopeRoots>,
}

struct Linker<'p> {
    files: &'p [LoadedFile],
    ids: &'p HashMap<super::program::FileKey, usize>,
    package_names: Vec<String>,
    graph: Graph,
    functions: Vec<Vec<FunctionId>>,
    by_name: HashMap<Unit, HashMap<String, Vec<FunctionId>>>,
    in_unit: HashMap<Unit, Vec<FunctionId>>,
    fallbacks: HashMap<usize, FunctionId>,
}

pub fn link(program: &Program, roots: &BTreeSet<String>) -> Linked {
    let mut linker = Linker {
        files: &program.files,
        ids: &program.ids,
        package_names: program.packages().names(),
        graph: Graph::default(),
        functions: Vec::new(),
        by_name: HashMap::new(),
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
            for definition in &file.index.definitions {
                let id = self.graph.add_function(Function {
                    name: format!("{}.{}", stem(&file.display), definition.name),
                    owner: file.owner.clone(),
                    path: file.display.clone(),
                });
                self.by_name
                    .entry(unit.clone())
                    .or_default()
                    .entry(definition.simple.clone())
                    .or_default()
                    .push(id);
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
                for target in self.targets(file, &item_use.target) {
                    self.graph.add_edge(
                        from,
                        Edge {
                            line: item_use.line,
                            target,
                        },
                    );
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
            self.graph.add_edge(
                from,
                Edge {
                    line: site.line,
                    target: Target::Sink(sink),
                },
            );
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
            self.graph.add_edge(
                init,
                Edge {
                    line: load.line,
                    target: Target::Function(target_init),
                },
            );
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
