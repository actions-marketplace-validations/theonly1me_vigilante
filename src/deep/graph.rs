use std::collections::{HashMap, HashSet};

use super::program::ArgumentFacts;
use crate::Power;

pub type FunctionId = usize;
pub type SinkId = usize;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Owner {
    Repository { scope: String },
    Dependency { package: String },
}

pub struct Function {
    pub name: String,
    pub owner: Owner,
    pub path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Sink {
    pub name: String,
    pub power: Power,
    pub rule: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    Function(FunctionId),
    Sink(SinkId),
}

#[derive(Clone, Copy, Debug)]
pub struct Edge {
    pub line: usize,
    pub target: Target,
}

#[derive(Clone, Copy, Debug)]
pub struct Call {
    pub from: FunctionId,
    pub line: usize,
    pub target: Target,
    pub facts: Option<ArgumentFacts>,
}

#[derive(Default)]
pub struct Graph {
    functions: Vec<Function>,
    edges: Vec<Vec<Edge>>,
    sinks: Vec<Sink>,
    sink_ids: HashMap<Sink, SinkId>,
    linked: HashSet<(FunctionId, Target)>,
    own_calls: Vec<Call>,
    own_call_keys: HashSet<(FunctionId, Target, usize, Option<ArgumentFacts>)>,
}

impl Graph {
    pub fn add_function(&mut self, function: Function) -> FunctionId {
        self.functions.push(function);
        self.edges.push(Vec::new());
        self.functions.len() - 1
    }

    pub fn add_sink(&mut self, sink: Sink) -> SinkId {
        if let Some(id) = self.sink_ids.get(&sink) {
            return *id;
        }
        self.sinks.push(sink.clone());
        self.sink_ids.insert(sink, self.sinks.len() - 1);
        self.sinks.len() - 1
    }

    pub fn add_call(&mut self, call: Call) {
        let is_own = matches!(self.functions[call.from].owner, Owner::Repository { .. });
        if is_own
            && self
                .own_call_keys
                .insert((call.from, call.target, call.line, call.facts))
        {
            self.own_calls.push(call);
        }
        if self.linked.insert((call.from, call.target)) {
            self.edges[call.from].push(Edge {
                line: call.line,
                target: call.target,
            });
        }
    }

    pub fn function(&self, id: FunctionId) -> &Function {
        &self.functions[id]
    }

    pub fn edges(&self, id: FunctionId) -> &[Edge] {
        &self.edges[id]
    }

    pub fn sink(&self, id: SinkId) -> &Sink {
        &self.sinks[id]
    }

    pub fn own_calls(&self) -> &[Call] {
        &self.own_calls
    }

    pub fn function_count(&self) -> usize {
        self.functions.len()
    }

    pub fn powers(&self) -> Vec<Power> {
        let mut powers: Vec<Power> = self.sinks.iter().map(|sink| sink.power.clone()).collect();
        powers.sort();
        powers.dedup();
        powers
    }
}
