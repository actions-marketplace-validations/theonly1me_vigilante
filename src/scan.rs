use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::scope::{Scopes, ScopesInput};
use crate::surfaces;
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
    })
    .scan()
}

impl Source {
    pub(crate) fn open(&self, repository: &Path) -> Result<Box<dyn Tree>, Error> {
        match self {
            Self::WorkingDirectory => Ok(Box::new(DirectoryTree::new(repository)?)),
            Self::Commit(reference) => {
                let git = Git::new(repository);
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
}

impl<'a> Scanner<'a> {
    pub fn new(input: ScannerInput<'a>) -> Self {
        let scopes = Scopes::new(ScopesInput {
            paths: input.tree.paths(),
            config: input.config,
        });
        Self {
            tree: input.tree,
            config: input.config,
            scopes,
        }
    }

    pub fn scan(&self) -> Result<PowerMap, Error> {
        let wanted: Vec<&str> = self
            .tree
            .paths()
            .iter()
            .map(String::as_str)
            .filter(|path| surfaces::handles(path) && !self.config.is_excluded(path))
            .collect();
        let mut map = PowerMap::default();
        for file in self.tree.read(&wanted)? {
            let source = SourceFile {
                path: &file.path,
                text: &file.text,
            };
            for detection in surfaces::detect(source)? {
                let surface = detection.power.capability.surface();
                let scope_path = match surface {
                    Surface::Code => self.scopes.code_scope_of(&file.path),
                    _ => file.path.clone(),
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
