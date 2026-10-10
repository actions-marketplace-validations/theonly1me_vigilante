use std::path::Path;

use crate::Error;
use crate::tree::{DirectoryTree, Git, Tree};

#[derive(Clone, Debug)]
pub enum Source {
    WorkingDirectory,
    Commit(String),
}

#[derive(Clone, Copy, Debug)]
pub enum Location<'a> {
    WorkingDirectory(&'a Path),
    Commit {
        repository: &'a Path,
        commit: &'a str,
    },
}

pub struct Located<'a> {
    pub tree: Box<dyn Tree>,
    repository: &'a Path,
    commit: Option<String>,
}

impl Located<'_> {
    pub fn location(&self) -> Location<'_> {
        match &self.commit {
            Some(commit) => Location::Commit {
                repository: self.repository,
                commit,
            },
            None => Location::WorkingDirectory(self.repository),
        }
    }
}

impl Source {
    pub(crate) fn locate<'a>(&self, repository: &'a Path) -> Result<Located<'a>, Error> {
        match self {
            Self::WorkingDirectory => Ok(Located {
                tree: Box::new(DirectoryTree::new(repository)?),
                repository,
                commit: None,
            }),
            Self::Commit(reference) => {
                let git = Git::open(repository)?;
                let commit = git.resolve(reference)?;
                Ok(Located {
                    tree: Box::new(git.tree(&commit)?),
                    repository,
                    commit: Some(commit),
                })
            }
        }
    }
}
