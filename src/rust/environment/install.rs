use std::fs;
use std::path::{Path, PathBuf};

use super::download::Planned;
use super::{Installed, RustEnvironment};
use crate::Error;
use crate::deep::Skip;
use crate::registry::{FetchRequest, Fetched, PackageId, folder_files};
use crate::rust::cargo::{LockedPackage, read_manifest};

const ECOSYSTEM: &str = "crates";
const MANIFEST: &str = "Cargo.toml";

pub struct Install<'a> {
    pub package: &'a LockedPackage,
    pub lock: usize,
}

pub fn is_kept(path: &str) -> bool {
    path.ends_with(".rs") || path.rsplit('/').next() == Some(MANIFEST)
}

impl RustEnvironment<'_> {
    pub(super) fn install(&mut self, install: Install) -> Result<Option<usize>, Error> {
        let Install { package, lock } = install;
        let key = (package.name.clone(), package.version.clone());
        if let Some(known) = self.install_of.get(&key) {
            return Ok(*known);
        }
        let source = self.lock_paths[lock].clone();
        let installed = match self.download_of(package)? {
            Planned::Skip(reason) => {
                self.skip(Skip { source, reason });
                None
            }
            Planned::Download(download) => {
                match self.registry.fetch(FetchRequest {
                    package: PackageId {
                        ecosystem: ECOSYSTEM,
                        name: &package.name,
                        version: &package.version,
                    },
                    download: &download,
                    keep: &is_kept,
                })? {
                    Fetched::Folder(folder) => Some(self.add(Added {
                        package,
                        folder: &folder,
                        lock,
                    })?),
                    Fetched::NotFound | Fetched::TooLarge => {
                        self.skip(Skip {
                            source,
                            reason: format!(
                                "`{} {}` could not be read from `{}`.",
                                package.name, package.version, download.url
                            ),
                        });
                        None
                    }
                }
            }
        };
        self.install_of.insert(key, installed);
        Ok(installed)
    }
}

struct Added<'a> {
    package: &'a LockedPackage,
    folder: &'a Path,
    lock: usize,
}

impl RustEnvironment<'_> {
    fn add(&mut self, added: Added) -> Result<usize, Error> {
        let Added {
            package,
            folder,
            lock,
        } = added;
        let root = crate_folder(CrateFolder {
            folder,
            name: &package.name,
        })
        .unwrap_or_else(|| folder.to_path_buf());
        let manifest = fs::read_to_string(root.join(MANIFEST))
            .ok()
            .and_then(|text| read_manifest(&text))
            .unwrap_or_default();
        self.installed.push(Installed {
            name: package.name.clone(),
            version: package.version.clone(),
            files: folder_files(&root)?,
            folder: root,
            manifest,
            lock,
        });
        Ok(self.installed.len() - 1)
    }
}

struct CrateFolder<'a> {
    folder: &'a Path,
    name: &'a str,
}

fn crate_folder(wanted: CrateFolder) -> Option<PathBuf> {
    let CrateFolder { folder, name } = wanted;
    let mut manifests: Vec<String> = folder_files(folder)
        .ok()?
        .into_iter()
        .filter(|path| path.rsplit('/').next() == Some(MANIFEST))
        .collect();
    manifests.sort_by_key(|path| path.matches('/').count());
    manifests.into_iter().find_map(|path| {
        let manifest = read_manifest(&fs::read_to_string(folder.join(&path)).ok()?)?;
        (manifest.name.as_deref() == Some(name))
            .then(|| folder.join(path.rsplit_once('/').map_or("", |(parent, _)| parent)))
    })
}
