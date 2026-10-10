use std::collections::{BTreeMap, HashMap, VecDeque};

use super::graph::{Function, FunctionId, Graph, SinkId, Target};
use crate::Power;

#[derive(Clone, Copy, Debug)]
pub struct Hop {
    pub function: FunctionId,
    pub line: usize,
}

#[derive(Clone, Debug)]
pub struct Route {
    pub hops: Vec<Hop>,
    pub sink: SinkId,
}

pub struct Search<'a> {
    pub graph: &'a Graph,
    pub roots: &'a [FunctionId],
    pub allowed: &'a dyn Fn(&Function) -> bool,
}

pub struct Explored {
    pub routes: BTreeMap<Power, Route>,
    pub order: Vec<FunctionId>,
    parents: HashMap<FunctionId, Option<Hop>>,
}

impl Explored {
    pub fn hops_before(&self, function: FunctionId) -> Vec<Hop> {
        let mut hops = Vec::new();
        let mut current = function;
        while let Some(Some(parent)) = self.parents.get(&current) {
            hops.push(*parent);
            current = parent.function;
        }
        hops.reverse();
        hops
    }
}

pub fn explore(search: Search) -> Explored {
    let mut explored = Explored {
        routes: BTreeMap::new(),
        order: Vec::new(),
        parents: HashMap::new(),
    };
    let mut queue = VecDeque::new();
    for root in search.roots {
        if !explored.parents.contains_key(root) {
            explored.parents.insert(*root, None);
            queue.push_back(*root);
        }
    }
    while let Some(function) = queue.pop_front() {
        explored.order.push(function);
        for edge in search.graph.edges(function) {
            match edge.target {
                Target::Sink(sink) => {
                    let power = &search.graph.sink(sink).power;
                    if !explored.routes.contains_key(power) {
                        let mut hops = explored.hops_before(function);
                        hops.push(Hop {
                            function,
                            line: edge.line,
                        });
                        explored.routes.insert(power.clone(), Route { hops, sink });
                    }
                }
                Target::Function(next) => {
                    if !explored.parents.contains_key(&next)
                        && (search.allowed)(search.graph.function(next))
                    {
                        explored.parents.insert(
                            next,
                            Some(Hop {
                                function,
                                line: edge.line,
                            }),
                        );
                        queue.push_back(next);
                    }
                }
            }
        }
    }
    explored
}

pub struct Toward {
    next_edge: Vec<Option<usize>>,
}

impl Toward {
    pub fn reaches(&self, function: FunctionId) -> bool {
        self.next_edge[function].is_some()
    }

    pub fn route_from(&self, graph: &Graph, start: FunctionId) -> Option<Route> {
        let mut hops = Vec::new();
        let mut current = start;
        loop {
            let edge = graph.edges(current).get(self.next_edge[current]?)?;
            hops.push(Hop {
                function: current,
                line: edge.line,
            });
            match edge.target {
                Target::Sink(sink) => return Some(Route { hops, sink }),
                Target::Function(next) => current = next,
            }
        }
    }
}

pub fn toward_each_power(graph: &Graph) -> BTreeMap<Power, Toward> {
    let mut callers: Vec<Vec<(FunctionId, usize)>> = vec![Vec::new(); graph.function_count()];
    for caller in 0..graph.function_count() {
        for (index, edge) in graph.edges(caller).iter().enumerate() {
            if let Target::Function(callee) = edge.target {
                callers[callee].push((caller, index));
            }
        }
    }
    graph
        .powers()
        .into_iter()
        .map(|power| {
            let mut next_edge = vec![None; graph.function_count()];
            let mut queue = VecDeque::new();
            for (function, next) in next_edge.iter_mut().enumerate() {
                let direct = graph.edges(function).iter().position(|edge| {
                    matches!(edge.target, Target::Sink(sink) if graph.sink(sink).power == power)
                });
                if direct.is_some() {
                    *next = direct;
                    queue.push_back(function);
                }
            }
            while let Some(function) = queue.pop_front() {
                for (caller, index) in &callers[function] {
                    if next_edge[*caller].is_none() {
                        next_edge[*caller] = Some(*index);
                        queue.push_back(*caller);
                    }
                }
            }
            (power, Toward { next_edge })
        })
        .collect()
}
