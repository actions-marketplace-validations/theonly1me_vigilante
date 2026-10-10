mod agent;
mod ci;
mod install_hooks;

use std::collections::BTreeSet;

use agent::{AgentFile, JsonFile};
use install_hooks::{HookFile, HookPath};

use crate::document::Node;
use crate::{Detection, Error, SourceFile};

pub struct Surfaces<'a> {
    paths: &'a BTreeSet<String>,
}

impl<'a> Surfaces<'a> {
    pub fn new(paths: &'a BTreeSet<String>) -> Self {
        Self { paths }
    }

    pub fn handles(&self, path: &str) -> bool {
        ci::is_workflow(path) || agent::kind_of(path).is_some() || self.hook_kind_of(path).is_some()
    }

    pub fn detect(&self, file: SourceFile) -> Result<Vec<Detection>, Error> {
        if ci::is_workflow(file.path) {
            return Ok(ci::detect(&parse(file)?));
        }
        if let Some(kind) = agent::kind_of(file.path) {
            return match kind {
                AgentFile::CodexConfig => agent::detect_codex(file),
                _ => Ok(agent::detect_json(JsonFile {
                    kind,
                    document: &parse_json(file)?,
                })),
            };
        }
        match self.hook_kind_of(file.path) {
            Some(HookFile::PackageManifest) => {
                Ok(install_hooks::detect_package_manifest(&parse_json(file)?))
            }
            Some(HookFile::PreCommitConfig) => Ok(install_hooks::detect_pre_commit(&parse(file)?)),
            Some(kind) => Ok(install_hooks::detect_static(kind)),
            None => Ok(Vec::new()),
        }
    }

    fn hook_kind_of<'path>(&self, path: &'path str) -> Option<HookFile<'path>>
    where
        'a: 'path,
    {
        install_hooks::kind_of(HookPath {
            path,
            paths: self.paths,
        })
    }
}

fn parse(file: SourceFile) -> Result<Node, Error> {
    Node::parse(file.text)
        .map_err(|error| Error::new(format!("cannot parse {}: {error}", file.path)))
}

fn parse_json(file: SourceFile) -> Result<Node, Error> {
    Node::parse_json(file.text)
        .map_err(|error| Error::new(format!("cannot parse {}: {error}", file.path)))
}
