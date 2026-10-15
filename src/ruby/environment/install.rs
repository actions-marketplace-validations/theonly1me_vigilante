use std::fs;
use std::path::{Path, PathBuf};

use super::download::Planned;
use super::{InstalledGem, RubyEnvironment};
use crate::Error;
use crate::deep::Skip;
use crate::registry::{FetchRequest, Fetched, PackageId, folder_files};
use crate::ruby::bundler::{GemSource, LockedGem};

pub const ECOSYSTEM: &str = "rubygems";
const SOURCE_SUFFIX: &str = ".rb";
const GEM_SPECIFICATION: &str = ".gemspec";

pub struct Install<'a> {
    pub gem: &'a LockedGem,
    pub lock: usize,
}

pub fn is_kept(path: &str) -> bool {
    path.ends_with(SOURCE_SUFFIX) || path.ends_with(GEM_SPECIFICATION)
}

impl RubyEnvironment<'_> {
    pub(super) fn install(&mut self, install: Install) -> Result<Option<usize>, Error> {
        let Install { gem, lock } = install;
        let key = (lock, gem.name.clone());
        if let Some(known) = self.install_of.get(&key) {
            return Ok(*known);
        }
        let source = self.lock_paths[lock].clone();
        let installed = match self.download_of(gem)? {
            Planned::Skip(reason) => {
                self.skip(Skip { source, reason });
                None
            }
            Planned::Download(download) => match self.registry.fetch(FetchRequest {
                package: PackageId {
                    ecosystem: ECOSYSTEM,
                    name: &gem.name,
                    version: &gem.version,
                },
                download: &download,
                keep: &is_kept,
            })? {
                Fetched::Folder(folder) => {
                    let root = match gem.source {
                        GemSource::Git { .. } => gem_folder(GemFolder {
                            folder: &folder,
                            name: &gem.name,
                        })
                        .unwrap_or(folder),
                        _ => folder,
                    };
                    self.installed.push(InstalledGem {
                        name: gem.name.clone(),
                        files: folder_files(&root)?,
                        folder: root,
                        lock,
                    });
                    Some(self.installed.len() - 1)
                }
                Fetched::NotFound | Fetched::TooLarge => {
                    self.skip(Skip {
                        source,
                        reason: format!(
                            "`{} {}` could not be read from `{}`.",
                            gem.name, gem.version, download.url
                        ),
                    });
                    None
                }
            },
        };
        self.install_of.insert(key, installed);
        Ok(installed)
    }
}

struct GemFolder<'a> {
    folder: &'a Path,
    name: &'a str,
}

fn gem_folder(wanted: GemFolder) -> Option<PathBuf> {
    let GemFolder { folder, name } = wanted;
    let specification = format!("{name}{GEM_SPECIFICATION}");
    let mut matches: Vec<String> = folder_files(folder)
        .ok()?
        .into_iter()
        .filter(|path| path.rsplit('/').next() == Some(specification.as_str()))
        .collect();
    matches.sort_by_key(|path| path.matches('/').count());
    let path = matches.into_iter().next()?;
    let parent = path.rsplit_once('/').map_or("", |(parent, _)| parent);
    Some(folder.join(parent)).filter(|root| fs::metadata(root).is_ok())
}
