use std::fs;
use std::path::{Path, PathBuf};

use super::PackageId;
use crate::Error;

const COMPLETE_MARKER: &str = ".vigilante-complete";

pub struct Cache {
    root: PathBuf,
}

pub struct CacheFill<'a> {
    pub package: &'a PackageId<'a>,
    pub fill: &'a dyn Fn(&Path) -> Result<(), Error>,
}

impl Cache {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn folder(&self, package: &PackageId) -> PathBuf {
        self.root
            .join(escaped(package.ecosystem))
            .join(escaped(package.name))
            .join(escaped(package.version))
    }

    pub fn complete_folder(&self, package: &PackageId) -> Option<PathBuf> {
        let folder = self.folder(package);
        folder.join(COMPLETE_MARKER).is_file().then_some(folder)
    }

    pub fn fill(&self, request: CacheFill) -> Result<PathBuf, Error> {
        let folder = self.folder(request.package);
        let parent = folder
            .parent()
            .ok_or_else(|| Error::new("the cache folder has no parent"))?;
        fs::create_dir_all(parent).map_err(cache_error)?;
        let staging = tempfile::Builder::new()
            .prefix(".vigilante-fill-")
            .tempdir_in(parent)
            .map_err(cache_error)?;
        (request.fill)(staging.path())?;
        fs::write(staging.path().join(COMPLETE_MARKER), b"").map_err(cache_error)?;
        let staged = staging.keep();
        if fs::rename(&staged, &folder).is_err() {
            fs::remove_dir_all(&staged).map_err(cache_error)?;
            return self.complete_folder(request.package).ok_or_else(|| {
                Error::new(format!("cannot store {} in the cache", folder.display()))
            });
        }
        Ok(folder)
    }
}

fn escaped(part: &str) -> String {
    let text: String = part
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' | '+' => character.to_string(),
            other => format!("%{:02X}", u32::from(other)),
        })
        .collect();
    match text.as_str() {
        "" | "." | ".." => format!("%{text}"),
        _ => text,
    }
}

fn cache_error(error: impl std::fmt::Display) -> Error {
    Error::new(format!("cannot use the package cache: {error}"))
}
