use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;

use super::{File, Tree};
use crate::Error;

const SYMBOLIC_LINK_MODE: &str = "120000";

#[derive(Clone, Debug)]
pub struct Git {
    repository: PathBuf,
}

#[derive(Clone, Copy, Debug)]
pub struct CommitPair<'a> {
    pub base: &'a str,
    pub head: &'a str,
}

pub struct GitTree {
    git: Git,
    objects: BTreeMap<String, String>,
    paths: BTreeSet<String>,
}

impl Git {
    pub fn new(repository: &Path) -> Self {
        Self {
            repository: repository.to_path_buf(),
        }
    }

    pub fn resolve(&self, reference: &str) -> Result<String, Error> {
        let revision = format!("{reference}^{{commit}}");
        self.run(&["rev-parse", "--verify", "--end-of-options", &revision])
            .map(first_line)
            .map_err(|_| Error::new(format!("`{reference}` is not a commit in this repository")))
    }

    pub fn merge_base(&self, pair: CommitPair) -> Result<String, Error> {
        self.run(&["merge-base", pair.base, pair.head])
            .map(first_line)
            .map_err(|_| Error::new("the base and head commits have no common ancestor"))
    }

    pub fn changed_paths(&self, pair: CommitPair) -> Result<BTreeSet<String>, Error> {
        let output = self.run(&[
            "diff",
            "--name-only",
            "-z",
            "--no-renames",
            "--no-ext-diff",
            "--no-textconv",
            pair.base,
            pair.head,
        ])?;
        Ok(split_at_nul(&output).collect())
    }

    pub fn tree(&self, commit: &str) -> Result<GitTree, Error> {
        let output = self.run(&["ls-tree", "-r", "-z", "--full-tree", commit])?;
        let objects: BTreeMap<String, String> = split_at_nul(&output)
            .filter_map(|entry| blob_of_entry(&entry))
            .collect();
        Ok(GitTree {
            git: self.clone(),
            paths: objects.keys().cloned().collect(),
            objects,
        })
    }

    fn command(&self) -> Command {
        let mut command = Command::new("git");
        command.arg("-C").arg(&self.repository).args([
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.quotePath=false",
        ]);
        command
    }

    fn run(&self, arguments: &[&str]) -> Result<Vec<u8>, Error> {
        let output = self
            .command()
            .args(arguments)
            .output()
            .map_err(|error| Error::new(format!("cannot run git: {error}")))?;
        if output.status.success() {
            return Ok(output.stdout);
        }
        Err(Error::new(format!(
            "git {} failed: {}",
            arguments.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

impl Tree for GitTree {
    fn paths(&self) -> &BTreeSet<String> {
        &self.paths
    }

    fn read(&self, paths: &[&str]) -> Result<Vec<File>, Error> {
        let wanted: Vec<(&str, &str)> = paths
            .iter()
            .filter_map(|path| Some((*path, self.objects.get(*path)?.as_str())))
            .collect();
        if wanted.is_empty() {
            return Ok(Vec::new());
        }
        let request: String = wanted
            .iter()
            .map(|(_, object)| format!("{object}\n"))
            .collect();
        let mut child = self
            .git
            .command()
            .args(["cat-file", "--batch"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|error| Error::new(format!("cannot run git cat-file: {error}")))?;
        let (Some(mut stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
            return Err(Error::new("git cat-file has no pipes"));
        };
        let writer = thread::spawn(move || stdin.write_all(request.as_bytes()));
        let mut reader = BufReader::new(stdout);
        let files = wanted
            .iter()
            .map(|(path, _)| {
                read_object(&mut reader).map(|contents| File {
                    path: path.to_string(),
                    text: String::from_utf8_lossy(&contents).into_owned(),
                })
            })
            .collect::<Result<Vec<File>, Error>>();
        if files.is_err() {
            child.kill().ok();
        }
        let written = writer
            .join()
            .map_err(|_| Error::new("git cat-file writer failed"))?;
        child
            .wait()
            .map_err(|error| Error::new(format!("git cat-file failed: {error}")))?;
        let files = files?;
        written.map_err(|error| Error::new(format!("cannot write to git cat-file: {error}")))?;
        Ok(files)
    }
}

fn read_object(reader: &mut impl BufRead) -> Result<Vec<u8>, Error> {
    let mut header = String::new();
    reader
        .read_line(&mut header)
        .map_err(|error| Error::new(format!("cannot read git cat-file output: {error}")))?;
    let size: usize = header
        .trim_end()
        .rsplit(' ')
        .next()
        .and_then(|size| size.parse().ok())
        .ok_or_else(|| Error::new(format!("unexpected git cat-file output: {header}")))?;
    let mut contents = vec![0; size + 1];
    reader
        .read_exact(&mut contents)
        .map_err(|error| Error::new(format!("cannot read git object: {error}")))?;
    contents.pop();
    Ok(contents)
}

fn blob_of_entry(entry: &str) -> Option<(String, String)> {
    let (header, path) = entry.split_once('\t')?;
    let mut fields = header.split(' ');
    let (mode, kind, object) = (fields.next()?, fields.next()?, fields.next()?);
    if kind != "blob" || mode == SYMBOLIC_LINK_MODE {
        return None;
    }
    Some((path.to_string(), object.to_string()))
}

fn split_at_nul(output: &[u8]) -> impl Iterator<Item = String> + '_ {
    output
        .split(|byte| *byte == 0)
        .filter(|part| !part.is_empty())
        .map(|part| String::from_utf8_lossy(part).into_owned())
}

fn first_line(output: Vec<u8>) -> String {
    String::from_utf8_lossy(&output).trim().to_string()
}
