mod project;
mod tool;

use std::collections::HashMap;
use std::fs;
use std::process::Command;

use crate::Error;
use crate::deep::program::{
    FileKey, Program, TypeCheck, Typed, TypedCall, TypedCalls, TypedTarget,
};
use crate::node::node_program;
use crate::registry::Registry;
use crate::scip::{Index, read_index};
use crate::tree::Tree;
use project::{CONFIG, ENVIRONMENT, INDEX, ProjectInput, write_project};

const NO_NODE: &str = "Node.js is not on PATH, so Vigilante resolved Python calls by name only.";
const NO_TOOL: &str =
    "Vigilante could not download scip-python 0.6.6, so it resolved Python calls by name only.";
const CALL_SUFFIX: &str = "().";
const LOCAL: &str = "local ";
const DESCRIPTOR_ENDS: &[char] = &['#', '/', '.'];

pub struct TypeCheckRequest<'a, 'p> {
    pub program: &'a Program<'p>,
    pub tree: &'a dyn Tree,
    pub registry: &'a Registry,
}

pub fn type_check(request: TypeCheckRequest) -> Result<TypeCheck, Error> {
    let Some(node) = node_program() else {
        return Ok(TypeCheck::NotChecked(NO_NODE.to_string()));
    };
    let Ok(Some(tool)) = tool::scip_python(request.registry) else {
        return Ok(TypeCheck::NotChecked(NO_TOOL.to_string()));
    };
    let project = write_project(ProjectInput {
        program: request.program,
        tree: request.tree,
    })?;
    let root = project.root.as_path();
    let output = Command::new(&node)
        .arg(&tool)
        .args(["index", "--quiet", "--project-name", "repository"])
        .args(["--project-version", "1", "--cwd"])
        .arg(root)
        .arg("--environment")
        .arg(root.join(ENVIRONMENT))
        .arg("--output")
        .arg(root.join(INDEX))
        .env_clear()
        .env("PATH", node.parent().unwrap_or(root))
        .output()
        .map_err(|error| Error::new(format!("could not start node: {error}")))?;
    let index = fs::read(root.join(INDEX))
        .ok()
        .filter(|_| output.status.success())
        .and_then(|bytes| read_index(&bytes));
    let Some(index) = index else {
        let reason = String::from_utf8_lossy(&output.stderr)
            .lines()
            .find(|line| !line.trim().is_empty() && !line.contains(CONFIG))
            .unwrap_or("no message")
            .to_string();
        return Ok(TypeCheck::NotChecked(format!(
            "scip-python stopped ({reason}), so Vigilante resolved Python calls by name only."
        )));
    };
    Ok(TypeCheck::Checked(typed_calls(Indexed {
        index: &index,
        keys: &project.keys,
    })))
}

struct Indexed<'a> {
    index: &'a Index,
    keys: &'a HashMap<String, FileKey>,
}

fn typed_calls(indexed: Indexed) -> TypedCalls {
    let Indexed { index, keys } = indexed;
    let mut definitions: HashMap<&str, TypedTarget> = HashMap::new();
    for document in &index.documents {
        let Some(file) = keys.get(&document.path) else {
            continue;
        };
        for occurrence in document
            .occurrences
            .iter()
            .filter(|item| item.is_definition)
        {
            if let Some(name) = called_name(&occurrence.symbol) {
                definitions.insert(
                    &occurrence.symbol,
                    TypedTarget {
                        file: file.clone(),
                        name: name.to_string(),
                        line: occurrence.line + 1,
                    },
                );
            }
        }
    }
    let mut calls = TypedCalls::default();
    for document in &index.documents {
        let Some(file) = keys.get(&document.path) else {
            continue;
        };
        for occurrence in document
            .occurrences
            .iter()
            .filter(|item| !item.is_definition)
        {
            let Some(target) = definitions.get(occurrence.symbol.as_str()) else {
                continue;
            };
            calls.add(TypedCall {
                file: file.clone(),
                line: occurrence.line + 1,
                callee: target.name.clone(),
                typed: Typed::Targets(vec![target.clone()]),
            });
        }
    }
    calls
}

fn called_name(symbol: &str) -> Option<&str> {
    if symbol.starts_with(LOCAL) {
        return None;
    }
    let path = symbol.strip_suffix(CALL_SUFFIX)?;
    let name = path.rsplit(DESCRIPTOR_ENDS).next()?.trim_matches('`');
    (!name.is_empty()).then_some(name)
}
