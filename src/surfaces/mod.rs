mod agent;
mod ci;
mod command;
mod download;
mod install_hooks;

use std::collections::BTreeSet;

use install_hooks::{HookFile, HookPath};

use crate::deep::RiskClass;
use crate::document::Node;
use crate::{Capability, Detection, Error, Power, SourceFile};

#[derive(Default)]
pub struct Detected {
    pub detections: Vec<Detection>,
    pub risks: Vec<SurfaceRisk>,
}

impl Detected {
    pub fn extend(&mut self, other: Detected) {
        self.detections.extend(other.detections);
        self.risks.extend(other.risks);
    }
}

impl From<Vec<Detection>> for Detected {
    fn from(detections: Vec<Detection>) -> Self {
        Self {
            detections,
            risks: Vec::new(),
        }
    }
}

pub struct SurfaceRisk {
    pub power: Power,
    pub class: RiskClass,
    pub identity: String,
    pub line: usize,
    pub call_path: Vec<String>,
}

pub struct Surfaces<'a> {
    paths: &'a BTreeSet<String>,
}

impl<'a> Surfaces<'a> {
    pub fn new(paths: &'a BTreeSet<String>) -> Self {
        Self { paths }
    }

    pub fn handles(&self, path: &str) -> bool {
        ci::is_workflow(path)
            || ci::is_composite_action(path)
            || agent::handles(path)
            || self.hook_kind_of(path).is_some()
    }

    pub fn detect(&self, file: SourceFile) -> Result<Detected, Error> {
        if ci::is_workflow(file.path) || ci::is_composite_action(file.path) {
            let readable = ci::without_expression_braces(file.text);
            let document = parse(SourceFile {
                path: file.path,
                text: &readable,
            })?;
            let workflow = ci::Workflow {
                document: &document,
                text: file.text,
            };
            return Ok(if ci::is_workflow(file.path) {
                ci::detect(workflow)
            } else {
                ci::detect_composite(workflow)
            });
        }
        if agent::handles(file.path) {
            return agent::detect(file).map(Detected::from);
        }
        let detections = match self.hook_kind_of(file.path) {
            Some(HookFile::PackageManifest) => {
                install_hooks::detect_package_manifest(&parse_json(file)?)
            }
            Some(HookFile::PreCommitConfig) => install_hooks::detect_pre_commit(&parse(file)?),
            Some(kind) => install_hooks::detect_static(kind),
            None => Vec::new(),
        };
        Ok(Detected::from(detections))
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

pub struct Placement<'a> {
    pub path: &'a str,
    pub capability: Capability,
}

pub fn scope_path_of(placement: Placement<'_>) -> &str {
    match placement.capability {
        Capability::ThirdPartyAction | Capability::RemoteScript => ci::WORKFLOWS,
        _ => placement.path,
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
