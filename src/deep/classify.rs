use std::collections::BTreeSet;
use std::iter;

use super::graph::{Function, FunctionId, Graph, Owner};
use super::search::{Route, Search, explore, toward_each_power};
use super::{Analysis, OwnPackagePath, Reach, ReachKind, note_kind_for_own_package};
use crate::{Evidence, Power, Scope, Sighting, Surface};

pub struct ScopeRoots {
    pub scope: String,
    pub roots: Vec<FunctionId>,
}

pub fn analyze(graph: &Graph, scopes: &[ScopeRoots]) -> Analysis {
    let towards = toward_each_power(graph);
    let mut packages_on_paths = BTreeSet::new();
    let mut analysis = Analysis::default();
    for scope in scopes {
        let reach_of = |reached: Reached| Reach {
            package: reached.package,
            through: reached.through,
            kind: reached.kind,
            power: reached.power.clone(),
            scope: code_scope(&scope.scope),
            evidence: evidence_of(graph, reached.route),
        };
        let own = explore(Search {
            graph,
            roots: &scope.roots,
            allowed: &is_repository,
        });
        for (power, route) in &own.routes {
            let kind = note_kind_for_own_package(OwnPackagePath {
                capability: power.capability,
                stays_in_repository: true,
            });
            match kind {
                None => analysis.sightings.push(Sighting {
                    scope: code_scope(&scope.scope),
                    power: power.clone(),
                    evidence: evidence_of(graph, route),
                }),
                Some(kind) => analysis.reaches.push(reach_of(Reached {
                    package: scope.scope.clone(),
                    through: scope.scope.clone(),
                    kind,
                    power,
                    route,
                })),
            }
        }
        let every = explore(Search {
            graph,
            roots: &scope.roots,
            allowed: &|_| true,
        });
        for (power, route) in &every.routes {
            if own.routes.contains_key(power) {
                continue;
            }
            let through = route
                .hops
                .iter()
                .find_map(|hop| dependency_of(graph.function(hop.function)))
                .unwrap_or(&scope.scope)
                .to_string();
            analysis.reaches.push(reach_of(Reached {
                package: scope.scope.clone(),
                through,
                kind: ReachKind::OnlyThroughOtherPackages,
                power,
                route,
            }));
        }
        for function in &every.order {
            let Some(package) = dependency_of(graph.function(*function)) else {
                continue;
            };
            for (power, toward) in &towards {
                if !toward.reaches(*function)
                    || packages_on_paths.contains(&(package.to_string(), power.clone()))
                {
                    continue;
                }
                let Some(rest) = toward.route_from(graph, *function) else {
                    continue;
                };
                packages_on_paths.insert((package.to_string(), power.clone()));
                let mut hops = every.hops_before(*function);
                hops.extend(rest.hops);
                analysis.reaches.push(reach_of(Reached {
                    package: package.to_string(),
                    through: package.to_string(),
                    kind: ReachKind::NewPackageOnPath,
                    power,
                    route: &Route {
                        hops,
                        sink: rest.sink,
                    },
                }));
            }
        }
    }
    analysis
}

struct Reached<'a> {
    package: String,
    through: String,
    kind: ReachKind,
    power: &'a Power,
    route: &'a Route,
}

fn is_repository(function: &Function) -> bool {
    matches!(function.owner, Owner::Repository { .. })
}

fn dependency_of(function: &Function) -> Option<&str> {
    match &function.owner {
        Owner::Dependency { package } => Some(package),
        Owner::Repository { .. } => None,
    }
}

fn code_scope(path: &str) -> Scope {
    Scope {
        surface: Surface::Code,
        path: path.to_string(),
    }
}

fn evidence_of(graph: &Graph, route: &Route) -> Evidence {
    let sink = graph.sink(route.sink);
    let (path, line) = route.hops.first().map_or((String::new(), 1), |hop| {
        (graph.function(hop.function).path.clone(), hop.line)
    });
    Evidence {
        path,
        line,
        rule: sink.rule.clone(),
        call_path: route
            .hops
            .iter()
            .map(|hop| graph.function(hop.function).name.clone())
            .chain(iter::once(sink.name.clone()))
            .collect(),
    }
}
