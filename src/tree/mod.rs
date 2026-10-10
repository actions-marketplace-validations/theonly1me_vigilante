mod directory;
mod git;

use std::collections::BTreeSet;

pub use directory::DirectoryTree;
pub use git::Git;

use crate::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct File {
    pub path: String,
    pub text: String,
}

pub trait Tree {
    fn paths(&self) -> &BTreeSet<String>;

    fn read(&self, paths: &[&str]) -> Result<Vec<File>, Error>;
}
