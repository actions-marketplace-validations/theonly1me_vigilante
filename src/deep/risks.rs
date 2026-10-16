use std::collections::{BTreeMap, HashMap, HashSet};

use super::graph::{FunctionId, Graph, Owner, Target};
use super::program::ArgumentFacts;
use super::{Risk, RiskClass};
use crate::{Capability, Evidence, Power, Scope, Surface};

const RISK_CAPABILITIES: [Capability; 5] = [
    Capability::Exec,
    Capability::Network,
    Capability::DynamicCode,
    Capability::Native,
    Capability::SystemCalls,
];
const TAINT_ROUNDS: usize = 8;

pub struct RisksInput<'a> {
    pub graph: &'a Graph,
    pub facts: &'a dyn Fn(&str, usize) -> Option<ArgumentFacts>,
}

struct Site {
    from: FunctionId,
    line: usize,
    sink: String,
    power: Power,
    rule: String,
}

pub fn risks(input: RisksInput) -> Vec<Risk> {
    let RisksInput { graph, facts } = input;
    let facts_at = |function: FunctionId, line: usize| {
        facts(&graph.function(function).path, line).unwrap_or(ArgumentFacts {
            constant: true,
            shell: false,
            untrusted: false,
            from_parameters: false,
        })
    };
    let tainted = tainted_functions(TaintInput {
        graph,
        facts_at: &facts_at,
    });
    let mut found: BTreeMap<(Scope, String), Risk> = BTreeMap::new();
    for site in sites(graph) {
        let Owner::Repository { scope } = &graph.function(site.from).owner else {
            continue;
        };
        let facts = facts_at(site.from, site.line);
        let class = if facts.untrusted || (facts.from_parameters && tainted.contains(&site.from)) {
            RiskClass::Untrusted
        } else if facts.shell && !facts.constant {
            RiskClass::Shell
        } else if !facts.constant {
            RiskClass::Dynamic
        } else {
            RiskClass::Constant
        };
        let function = graph.function(site.from);
        let risk = Risk {
            scope: Scope {
                surface: Surface::Code,
                path: scope.clone(),
            },
            identity: format!(
                "{}:{}:{}:{}",
                site.power,
                function.name,
                site.sink,
                class.name()
            ),
            power: site.power,
            class,
            evidence: Evidence {
                path: function.path.clone(),
                line: site.line,
                rule: site.rule,
                call_path: vec![function.name.clone(), site.sink],
            },
        };
        found
            .entry((risk.scope.clone(), risk.identity.clone()))
            .or_insert(risk);
    }
    found.into_values().collect()
}

fn sites(graph: &Graph) -> Vec<Site> {
    graph
        .own_calls()
        .iter()
        .filter_map(|call| {
            let Target::Sink(sink) = call.target else {
                return None;
            };
            let sink = graph.sink(sink);
            RISK_CAPABILITIES
                .contains(&sink.power.capability)
                .then(|| Site {
                    from: call.from,
                    line: call.line,
                    sink: sink.name.clone(),
                    power: sink.power.clone(),
                    rule: sink.rule.clone(),
                })
        })
        .collect()
}

struct TaintInput<'a> {
    graph: &'a Graph,
    facts_at: &'a dyn Fn(FunctionId, usize) -> ArgumentFacts,
}

fn tainted_functions(input: TaintInput) -> HashSet<FunctionId> {
    let TaintInput { graph, facts_at } = input;
    let calls: Vec<(FunctionId, FunctionId, usize)> = graph
        .own_calls()
        .iter()
        .filter_map(|call| match call.target {
            Target::Function(callee)
                if matches!(graph.function(callee).owner, Owner::Repository { .. }) =>
            {
                Some((call.from, callee, call.line))
            }
            _ => None,
        })
        .collect();
    let facts: HashMap<(FunctionId, usize), ArgumentFacts> = calls
        .iter()
        .map(|(from, _, line)| ((*from, *line), facts_at(*from, *line)))
        .collect();
    let mut tainted = HashSet::new();
    for _ in 0..TAINT_ROUNDS {
        let before = tainted.len();
        for (from, callee, line) in &calls {
            let facts = facts[&(*from, *line)];
            if facts.untrusted || (facts.from_parameters && tainted.contains(from)) {
                tainted.insert(*callee);
            }
        }
        if tainted.len() == before {
            break;
        }
    }
    tainted
}
