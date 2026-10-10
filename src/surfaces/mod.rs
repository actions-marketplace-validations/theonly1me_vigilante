mod agent;
mod ci;

use agent::{AgentFile, JsonFile};

use crate::document::Node;
use crate::{Detection, Error, SourceFile};

pub fn handles(path: &str) -> bool {
    ci::is_workflow(path) || agent::kind_of(path).is_some()
}

pub fn detect(file: SourceFile) -> Result<Vec<Detection>, Error> {
    if ci::is_workflow(file.path) {
        return Ok(ci::detect(&parse(file)?));
    }
    match agent::kind_of(file.path) {
        Some(AgentFile::CodexConfig) => agent::detect_codex(file),
        Some(kind) => Ok(agent::detect_json(JsonFile {
            kind,
            document: &parse_json(file)?,
        })),
        None => Ok(Vec::new()),
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
