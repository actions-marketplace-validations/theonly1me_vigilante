mod rule_files;
mod source;

use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::code::Rules;
use crate::config::{Config, GoEngine};
use crate::go::{DiscoverInput, GoModules, GoRequest, Reach};
use crate::scope::{Scopes, ScopesInput};
use crate::surfaces::{self, Placement, Surfaces};
use crate::tree::Tree;
use crate::{Error, Evidence, PowerMap, Scope, Sighting, SourceFile, Surface};
use rule_files::{RuleSources, rules_of};
pub use source::{Location, Source};

#[derive(Clone, Debug)]
pub struct ScanRequest {
    pub repository: PathBuf,
    pub source: Source,
}

pub fn scan(request: ScanRequest) -> Result<PowerMap, Error> {
    let located = request.source.locate(&request.repository)?;
    let config = Config::load(located.tree.as_ref())?;
    Scanner::new(ScannerInput {
        tree: located.tree.as_ref(),
        config: &config,
        location: located.location(),
    })?
    .scan(&Selection::Everything)
    .map(|scanned| scanned.map)
}

pub struct Scanned {
    pub map: PowerMap,
    pub reaches: Vec<Reach>,
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
                tree: input.tree,
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
                let is_wanted = self.surfaces.handles(path)
                    || (self.rules.handles(path) && is_selected_code(path) && !self.go_owns(path));
                is_wanted && !self.config.is_excluded(path)
            })
            .collect();
        let mut map = PowerMap::default();
        for file in self.tree.read(&wanted)? {
            let source = SourceFile {
                path: &file.path,
                text: &file.text,
            };
            let detections = self.surfaces.detect(source)?;
            for detection in detections
                .into_iter()
                .chain(self.rules.detect(source))
                .filter(|detection| !self.config.trusts(&detection.power))
            {
                let surface = detection.power.capability.surface();
                let scope_path = match surface {
                    Surface::Code => self.scopes.code_scope_of(&file.path),
                    _ => surfaces::scope_path_of(Placement {
                        path: &file.path,
                        capability: detection.power.capability,
                    })
                    .to_string(),
                };
                map.insert(Sighting {
                    scope: Scope {
                        surface,
                        path: scope_path,
                    },
                    power: detection.power,
                    evidence: Evidence {
                        path: file.path.clone(),
                        line: detection.line,
                        rule: detection.rule,
                        call_path: Vec::new(),
                    },
                });
            }
        }
        let mut reaches = Vec::new();
        if let Some(go) = &self.go {
            let changed = match selection {
                Selection::Everything => None,
                Selection::Changes(changes) => Some(&changes.paths),
            };
            let analysis = go.analyze(GoRequest {
                changed,
                location: self.location,
                config: self.config,
            })?;
            for sighting in analysis.sightings {
                map.insert(sighting);
            }
            reaches = analysis.reaches;
        }
        Ok(Scanned { map, reaches })
    }

    fn go_owns(&self, path: &str) -> bool {
        self.go.as_ref().is_some_and(|go| go.owns(path))
    }
}
