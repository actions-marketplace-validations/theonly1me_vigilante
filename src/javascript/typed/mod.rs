mod compiler;
mod input;
#[cfg(test)]
mod tests;

use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;

use serde::Deserialize;

use crate::Error;
use crate::deep::program::{Program, TypeCheck, Typed, TypedCall, TypedCalls, TypedTarget};
use crate::registry::Registry;
use crate::tree::Tree;
use input::{CheckerInput, VirtualFiles, checker_input};

const SCRIPT: &str = include_str!("calls.mjs");
const NO_NODE: &str =
    "Node.js is not on PATH, so Vigilante resolved JavaScript and TypeScript calls by name only.";
const NO_COMPILER: &str =
    "Vigilante could not download TypeScript 6.0.3, so it resolved calls by name only.";

pub struct TypeCheckRequest<'a, 'p> {
    pub program: &'a Program<'p>,
    pub tree: &'a dyn Tree,
    pub registry: &'a Registry,
}

#[derive(Deserialize)]
struct Line {
    file: String,
    line: usize,
    callee: String,
    #[serde(default)]
    targets: Vec<LineTarget>,
    #[serde(default)]
    unanalyzed: bool,
}

#[derive(Deserialize)]
struct LineTarget {
    file: String,
    name: String,
    line: usize,
}

pub fn type_check(request: TypeCheckRequest) -> Result<TypeCheck, Error> {
    if Command::new("node").arg("--version").output().is_err() {
        return Ok(TypeCheck::NotChecked(NO_NODE.to_string()));
    }
    let Ok(Some(compiler)) = compiler::typescript(request.registry) else {
        return Ok(TypeCheck::NotChecked(NO_COMPILER.to_string()));
    };
    let (input, files) = checker_input(CheckerInput {
        program: request.program,
        tree: request.tree,
        compiler: &compiler,
    })?;
    let mut child = match Command::new("node")
        .args(["--input-type=module", "--eval", SCRIPT])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => return Err(Error::new(format!("could not start node: {error}"))),
    };
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| Error::new("could not write to node"))?;
    let writer = thread::spawn(move || stdin.write_all(&input));
    let output = child
        .wait_with_output()
        .map_err(|error| Error::new(format!("node did not finish: {error}")))?;
    let written = writer
        .join()
        .map_err(|_| Error::new("the writer to node stopped"))?;
    if !output.status.success() || written.is_err() {
        let reason = String::from_utf8_lossy(&output.stderr)
            .lines()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("no message")
            .to_string();
        return Ok(TypeCheck::NotChecked(format!(
            "The TypeScript checker stopped ({reason}), so Vigilante resolved calls by name only."
        )));
    }
    Ok(TypeCheck::Checked(typed_calls(TypedOutput {
        stdout: &String::from_utf8_lossy(&output.stdout),
        files: &files,
    })))
}

struct TypedOutput<'a> {
    stdout: &'a str,
    files: &'a VirtualFiles,
}

fn typed_calls(output: TypedOutput) -> TypedCalls {
    let mut calls = TypedCalls::default();
    for line in output.stdout.lines().filter(|line| !line.trim().is_empty()) {
        let Ok(line) = serde_json::from_str::<Line>(line) else {
            continue;
        };
        let Some(file) = output.files.key_of(&line.file) else {
            continue;
        };
        let typed = if line.unanalyzed {
            Typed::Unanalyzed
        } else {
            Typed::Targets(
                line.targets
                    .into_iter()
                    .filter_map(|target| {
                        Some(TypedTarget {
                            file: output.files.key_of(&target.file)?,
                            name: target.name,
                            line: target.line,
                        })
                    })
                    .collect(),
            )
        };
        calls.add(TypedCall {
            file,
            line: line.line,
            callee: line.callee,
            typed,
        });
    }
    calls
}
