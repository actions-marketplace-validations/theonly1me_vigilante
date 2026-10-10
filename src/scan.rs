use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::code::Rules;
use crate::config::Config;
use crate::scope::{Scopes, ScopesInput};
use crate::surfaces::{self, Placement, Surfaces};
use crate::tree::{DirectoryTree, Git, Tree};
use crate::{Error, Evidence, PowerMap, Scope, Sighting, SourceFile, Surface};

#[derive(Clone, Debug)]
pub struct ScanRequest {
    pub repository: PathBuf,
    pub source: Source,
}

#[derive(Clone, Debug)]
pub enum Source {
    WorkingDirectory,
    Commit(String),
}

pub fn scan(request: ScanRequest) -> Result<PowerMap, Error> {
    let tree = request.source.open(&request.repository)?;
    let config = Config::load(tree.as_ref())?;
    Scanner::new(ScannerInput {
        tree: tree.as_ref(),
        config: &config,
    })?
    .scan(&Selection::Everything)
}

pub enum Selection {
    Everything,
    CodeScopes(BTreeSet<String>),
}

impl Source {
    pub(crate) fn open(&self, repository: &Path) -> Result<Box<dyn Tree>, Error> {
        match self {
            Self::WorkingDirectory => Ok(Box::new(DirectoryTree::new(repository)?)),
            Self::Commit(reference) => {
                let git = Git::open(repository)?;
                let commit = git.resolve(reference)?;
                Ok(Box::new(git.tree(&commit)?))
            }
        }
    }
}

pub struct ScannerInput<'a> {
    pub tree: &'a dyn Tree,
    pub config: &'a Config,
}

pub struct Scanner<'a> {
    tree: &'a dyn Tree,
    config: &'a Config,
    scopes: Scopes<'a>,
    surfaces: Surfaces<'a>,
    rules: Rules,
}

impl<'a> Scanner<'a> {
    pub fn new(input: ScannerInput<'a>) -> Result<Self, Error> {
        let scopes = Scopes::new(ScopesInput {
            paths: input.tree.paths(),
            config: input.config,
        });
        Ok(Self {
            tree: input.tree,
            config: input.config,
            scopes,
            surfaces: Surfaces::new(input.tree.paths()),
            rules: rules_of(input)?,
        })
    }

    pub fn code_scope_of(&self, path: &str) -> String {
        self.scopes.code_scope_of(path)
    }

    pub fn scan(&self, selection: &Selection) -> Result<PowerMap, Error> {
        let is_selected_code = |path: &str| match selection {
            Selection::Everything => true,
            Selection::CodeScopes(scopes) => scopes.contains(&self.code_scope_of(path)),
        };
        let wanted: Vec<&str> = self
            .tree
            .paths()
            .iter()
            .map(String::as_str)
            .filter(|path| {
                let is_wanted = self.surfaces.handles(path)
                    || (self.rules.handles(path) && is_selected_code(path));
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
                    },
                });
            }
        }
        Ok(map)
    }
}

fn rules_of(input: ScannerInput) -> Result<Rules, Error> {
    let mut rules = Rules::built_in()?;
    let folders: Vec<String> = input
        .config
        .rule_paths
        .iter()
        .map(|folder| format!("{}/", folder.trim_end_matches('/')))
        .collect();
    let rule_files: Vec<&str> = input
        .tree
        .paths()
        .iter()
        .map(String::as_str)
        .filter(|path| {
            folders
                .iter()
                .any(|folder| path.starts_with(folder.as_str()))
                && (path.ends_with(".yml") || path.ends_with(".yaml"))
        })
        .collect();
    for file in input.tree.read(&rule_files)? {
        rules.add(SourceFile {
            path: &file.path,
            text: &file.text,
        })?;
    }
    Ok(rules)
}
