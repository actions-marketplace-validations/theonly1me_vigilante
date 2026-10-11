mod compiler;
mod input;
#[cfg(test)]
mod tests;

use std::process::Command;
use std::time::{Duration, Instant};

use serde::Deserialize;

use crate::Error;
use crate::deep::program::{Program, TypeCheck, Typed, TypedCall, TypedCalls, TypedTarget};
use crate::process::{Finished, LimitedRun, run_limited};
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
    pub limit: Duration,
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
    let finished = run_limited(LimitedRun {
        command: Command::new("node").args(["--input-type=module", "--eval", SCRIPT]),
        input,
        deadline: Instant::now() + request.limit,
    })
    .map_err(|error| Error::new(format!("could not run node: {error}")))?;
    let Finished::Exited(output) = finished else {
        return Ok(TypeCheck::NotChecked(format!(
            "The TypeScript checker reached the time limit of type_check_seconds = {}, so Vigilante resolved calls by name only.",
            request.limit.as_secs()
        )));
    };
    if !output.status.success() {
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
