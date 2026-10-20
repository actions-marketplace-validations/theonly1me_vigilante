mod deep;
mod placing;
mod report;
mod rule_files;
mod source;
mod test_code;

use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::code::Rules;
use crate::config::{Config, GoEngine, TestPolicy};
use crate::deep::{Reach, Risk, Skip};
use crate::go::{DiscoverInput, GoModules};
use crate::scope::{Scopes, ScopesInput};
use crate::surfaces::Surfaces;
use crate::tree::Tree;
use crate::{Error, PowerMap, Sighting, SourceFile};
use placing::Placing;
pub use report::{ScanReport, scan_report};
use rule_files::{RuleSources, rules_of};
pub use source::{Location, Source};
use test_code::{TestFile, test_sightings};

#[derive(Clone, Debug)]
pub struct ScanRequest {
    pub repository: PathBuf,
    pub source: Source,
}

pub fn scan(request: ScanRequest) -> Result<PowerMap, Error> {
    scan_report(request).map(|report| report.map)
}

pub struct Scanned {
    pub map: PowerMap,
    pub tests: Vec<Sighting>,
    pub examples: Vec<Sighting>,
    pub risks: Vec<Risk>,
    pub reaches: Vec<Reach>,
    pub skipped: Vec<Skip>,
}

pub enum Selection {
    Everything,
    Changes(Changes),
}

pub struct Changes {
    pub code_scopes: BTreeSet<String>,
    pub paths: BTreeSet<String>,
}

pub struct ScannerInput<'a> {
    pub tree: &'a dyn Tree,
    pub rules_tree: &'a dyn Tree,
    pub config: &'a Config,
    pub location: Location<'a>,
}

pub struct Scanner<'a> {
    tree: &'a dyn Tree,
    config: &'a Config,
    location: Location<'a>,
    scopes: Scopes<'a>,
    surfaces: Surfaces<'a>,
    rules: Rules,
    go: Option<GoModules>,
}

impl<'a> Scanner<'a> {
    pub fn new(input: ScannerInput<'a>) -> Result<Self, Error> {
        let go = match input.config.go_engine {
            GoEngine::Capslock => Some(GoModules::discover(DiscoverInput {
                tree: input.tree,
                config: input.config,
            })?),
            GoEngine::Patterns => None,
        };
        Ok(Self {
            tree: input.tree,
            config: input.config,
            location: input.location,
            scopes: Scopes::new(ScopesInput {
                paths: input.tree.paths(),
                config: input.config,
            }),
            surfaces: Surfaces::new(input.tree.paths()),
            rules: rules_of(RuleSources {
                tree: input.rules_tree,
                config: input.config,
            })?,
            go,
        })
    }

    pub fn code_scope_of(&self, path: &str) -> String {
        self.scopes.code_scope_of(path)
    }

    pub fn scan(&self, selection: &Selection) -> Result<Scanned, Error> {
        let is_selected_code = |path: &str| match selection {
            Selection::Everything => true,
            Selection::Changes(changes) => changes.code_scopes.contains(&self.code_scope_of(path)),
        };
        let wanted: Vec<&str> = self
            .tree
            .paths()
            .iter()
            .map(String::as_str)
            .filter(|path| {
                if self.config.is_excluded(path) {
                    return false;
                }
                let is_code = self.rules.handles(path) && is_selected_code(path);
                if self.config.is_test(path) {
                    return is_code;
                }
                self.surfaces.handles(path) || (is_code && !self.deep_owns(path))
            })
            .collect();
        let mut map = PowerMap::default();
        let mut tests = Vec::new();
        let mut risks = Vec::new();
        for file in self.tree.read(&wanted)? {
            let source = SourceFile {
                path: &file.path,
                text: &file.text,
            };
            if self.config.is_test(&file.path) {
                let found = test_sightings(TestFile {
                    rules: &self.rules,
                    scope: self.scopes.code_scope_of(source.path),
                    source,
                });
                match self.config.tests {
                    TestPolicy::Lock => found.into_iter().for_each(|sighting| map.insert(sighting)),
                    TestPolicy::Report => tests.extend(found),
                }
                continue;
            }
            let detected = self.surfaces.detect(source)?;
            let code = if self.deep_owns(&file.path) {
                Vec::new()
            } else {
                self.rules.detect(source)
            };
            let placing = Placing {
                scopes: &self.scopes,
                path: &file.path,
            };
            for detection in detected
                .detections
                .into_iter()
                .chain(code)
                .filter(|detection| !self.config.trusts(&detection.power))
            {
                map.insert(placing.sighting_of(detection));
            }
            risks.extend(
                detected
                    .risks
                    .into_iter()
                    .filter(|risk| !self.config.trusts(&risk.power))
                    .map(|risk| placing.risk_of(risk)),
            );
        }
        let analysis = self.deep(selection)?;
        for sighting in analysis.sightings {
            map.insert(sighting);
        }
        risks.extend(analysis.risks);
        Ok(Scanned {
            map,
            tests,
            examples: analysis.examples,
            risks,
            reaches: analysis.reaches,
            skipped: analysis.skipped,
        })
    }

    fn deep_owns(&self, path: &str) -> bool {
        self.go.as_ref().is_some_and(|go| go.owns(path))
            || (self.config.javascript_engine.follows_calls() && crate::javascript::owns(path))
            || (self.config.python_engine.follows_calls() && crate::python::owns(path))
            || (self.config.java_engine.follows_calls() && crate::java::owns(path))
            || (self.config.rust_engine.follows_calls() && crate::rust::owns(path))
            || (self.config.csharp_engine.follows_calls() && crate::csharp::owns(path))
            || (self.config.ruby_engine.follows_calls() && crate::ruby::owns(path))
    }
}
