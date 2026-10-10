mod ci;

use crate::document::Node;
use crate::{Detection, Error, SourceFile};

pub fn handles(path: &str) -> bool {
    ci::is_workflow(path)
}

pub fn detect(file: SourceFile) -> Result<Vec<Detection>, Error> {
    if ci::is_workflow(file.path) {
        return Ok(ci::detect(&parse(file)?));
    }
    Ok(Vec::new())
}

fn parse(file: SourceFile) -> Result<Node, Error> {
    Node::parse(file.text)
        .map_err(|error| Error::new(format!("cannot parse {}: {error}", file.path)))
}
