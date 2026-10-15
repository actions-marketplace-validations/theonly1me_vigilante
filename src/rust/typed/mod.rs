use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;
use std::time::Duration;

use tempfile::TempDir;

use crate::Error;
use crate::container::{ContainerRequest, ContainerRun, OUTPUT, run_contained};
use crate::deep::program::{FileKey, Program, Root, TypeCheck, TypedCalls};
use crate::repository_path::parent_folder;
use crate::scip::{CallsInput, read_index, typed_calls};
use crate::tree::Tree;

const IMAGE: &str =
    "rust:1.99-slim@sha256:2752b332db73fdbb7dc576f06c82ed1f312005784ef913d7e04a28f5f55dc581";
const PREPARE: &str =
    "rustup component add rust-analyzer rust-src > /dev/null 2>&1 && cargo fetch --locked";
const INDEX: &str = "index.scip";
const LOCKFILE: &str = "Cargo.lock";

pub struct TypeCheckRequest<'a, 'p> {
    pub program: &'a Program<'p>,
    pub tree: &'a dyn Tree,
    pub limit: Duration,
}

pub fn type_check(request: TypeCheckRequest) -> Result<TypeCheck, Error> {
    let TypeCheckRequest {
        program,
        tree,
        limit,
    } = request;
    let mut workspaces: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for file in &program.files {
        if file.key.root != Root::Repository {
            continue;
        }
        if let Some(workspace) = workspace_of(WorkspaceQuery {
            tree,
            path: &file.key.path,
        }) {
            workspaces
                .entry(workspace)
                .or_default()
                .push(file.key.path.clone());
        }
    }
    let mut calls = TypedCalls::default();
    for (workspace, own) in workspaces {
        match built_calls(Workspace {
            tree,
            folder: &workspace,
            own: &own,
            limit,
        })? {
            TypeCheck::Checked(found) => calls.extend(found),
            not_checked => return Ok(not_checked),
        }
    }
    Ok(TypeCheck::Checked(calls))
}

struct WorkspaceQuery<'a> {
    tree: &'a dyn Tree,
    path: &'a str,
}

fn workspace_of(query: WorkspaceQuery) -> Option<String> {
    let mut folder = parent_folder(query.path);
    loop {
        let lockfile = within(WithinFolder {
            folder,
            path: LOCKFILE,
        });
        if query.tree.paths().contains(&lockfile) {
            return Some(folder.to_string());
        }
        if folder.is_empty() {
            return None;
        }
        folder = parent_folder(folder);
    }
}

struct Workspace<'a> {
    tree: &'a dyn Tree,
    folder: &'a str,
    own: &'a [String],
    limit: Duration,
}

fn built_calls(workspace: Workspace) -> Result<TypeCheck, Error> {
    let Workspace {
        tree,
        folder,
        own,
        limit,
    } = workspace;
    let source = TempDir::new().map_err(io)?;
    let output = TempDir::new().map_err(io)?;
    let prefix = if folder.is_empty() {
        String::new()
    } else {
        format!("{folder}/")
    };
    let paths: Vec<&str> = tree
        .paths()
        .iter()
        .map(String::as_str)
        .filter(|path| path.starts_with(&prefix))
        .collect();
    for file in tree.read(&paths)? {
        write(WriteFile {
            path: &source.path().join(&file.path[prefix.len()..]),
            bytes: file.text.as_bytes(),
        })?;
    }
    let build = format!("rust-analyzer scip . --output {OUTPUT}/{INDEX}");
    let run = run_contained(ContainerRequest {
        image: IMAGE,
        source: &fs::canonicalize(source.path()).map_err(io)?,
        output: &fs::canonicalize(output.path()).map_err(io)?,
        prepare: PREPARE,
        build: &build,
        timeout: limit,
    })?;
    let reason = match run {
        ContainerRun::Built => None,
        ContainerRun::NotBuilt(reason) => Some(reason),
        ContainerRun::OutOfTime => Some(format!(
            "it reached the time limit of type_check_seconds = {}",
            limit.as_secs()
        )),
    };
    if let Some(reason) = reason {
        return Ok(TypeCheck::NotChecked(format!(
            "Vigilante did not build this Rust code ({reason}), so it resolved Rust calls by name only."
        )));
    }
    let Some(index) = fs::read(output.path().join(INDEX))
        .ok()
        .and_then(|bytes| read_index(&bytes))
    else {
        return Ok(TypeCheck::NotChecked(
            "rust-analyzer wrote no index, so Vigilante resolved Rust calls by name only."
                .to_string(),
        ));
    };
    let keys: HashMap<String, FileKey> = own
        .iter()
        .map(|path| {
            (
                path[prefix.len()..].to_string(),
                FileKey {
                    root: Root::Repository,
                    path: path.clone(),
                },
            )
        })
        .collect();
    Ok(TypeCheck::Checked(typed_calls(CallsInput {
        index: &index,
        keys: &keys,
        external: &|_| None,
    })))
}

struct WithinFolder<'a> {
    folder: &'a str,
    path: &'a str,
}

fn within(input: WithinFolder) -> String {
    if input.folder.is_empty() {
        input.path.to_string()
    } else {
        format!("{}/{}", input.folder, input.path)
    }
}

fn io(error: std::io::Error) -> Error {
    Error::new(format!("could not prepare the Rust build: {error}"))
}

struct WriteFile<'a> {
    path: &'a Path,
    bytes: &'a [u8],
}

fn write(file: WriteFile) -> Result<(), Error> {
    if let Some(parent) = file.path.parent() {
        fs::create_dir_all(parent).map_err(io)?;
    }
    fs::write(file.path, file.bytes).map_err(io)
}
