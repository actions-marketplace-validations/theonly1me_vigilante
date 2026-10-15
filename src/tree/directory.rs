use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;

use super::{File, Tree};
use crate::Error;

pub struct DirectoryTree {
    root: PathBuf,
    paths: BTreeSet<String>,
}

impl DirectoryTree {
    pub fn new(root: &Path) -> Result<Self, Error> {
        let walk = WalkBuilder::new(root)
            .hidden(false)
            .ignore(false)
            .git_global(false)
            .filter_entry(|entry| entry.file_name() != ".git")
            .build();
        let mut paths = BTreeSet::new();
        for entry in walk {
            let entry = entry
                .map_err(|error| Error::new(format!("cannot walk {}: {error}", root.display())))?;
            if !entry.file_type().is_some_and(|kind| kind.is_file()) {
                continue;
            }
            if let Ok(relative) = entry.path().strip_prefix(root) {
                paths.insert(slash_path(relative));
            }
        }
        Ok(Self {
            root: root.to_path_buf(),
            paths,
        })
    }
}

impl Tree for DirectoryTree {
    fn paths(&self) -> &BTreeSet<String> {
        &self.paths
    }

    fn read(&self, paths: &[&str]) -> Result<Vec<File>, Error> {
        paths
            .iter()
            .filter(|path| self.paths.contains(**path))
            .map(|path| {
                fs::read(self.root.join(path))
                    .map(|contents| File {
                        path: path.to_string(),
                        text: String::from_utf8_lossy(&contents).into_owned(),
                    })
                    .map_err(|error| Error::new(format!("cannot read {path}: {error}")))
            })
            .collect()
    }
}

fn slash_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}
