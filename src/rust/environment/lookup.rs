use super::RustEnvironment;
use super::download::Planned;
use super::install::{Install, is_kept};
use crate::Error;
use crate::registry::Prefetch;
use crate::repository_path::{Folder, parent_folder};
use crate::rust::cargo::{LockedPackage, read_lockfile};

const LOCKFILE: &str = "Cargo.lock";
const ECOSYSTEM: &str = "crates";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CrateId {
    Own(usize),
    Package(usize),
}

pub enum Located {
    Own(usize),
    Package(usize),
    Unknown,
    NoLockfile,
    Skipped,
}

pub struct DependencyLookup<'a> {
    pub id: CrateId,
    pub name: &'a str,
}

enum Lookup {
    Own(usize),
    Locked { package: LockedPackage, lock: usize },
    Unknown,
    NoLockfile,
}

impl RustEnvironment<'_> {
    pub fn dependency(&mut self, wanted: DependencyLookup) -> Result<Located, Error> {
        Ok(match self.lookup(wanted)? {
            Lookup::Own(position) => Located::Own(position),
            Lookup::Unknown => Located::Unknown,
            Lookup::NoLockfile => Located::NoLockfile,
            Lookup::Locked { package, lock } => match self.install(Install {
                package: &package,
                lock,
            })? {
                Some(installed) => Located::Package(installed),
                None => Located::Skipped,
            },
        })
    }

    pub fn planned(&mut self, wanted: DependencyLookup) -> Option<Prefetch> {
        let Ok(Lookup::Locked { package, .. }) = self.lookup(wanted) else {
            return None;
        };
        if self
            .install_of
            .contains_key(&(package.name.clone(), package.version.clone()))
        {
            return None;
        }
        let Ok(Planned::Download(download)) = self.download_of(&package) else {
            return None;
        };
        Some(Prefetch {
            ecosystem: ECOSYSTEM,
            name: package.name,
            version: package.version,
            download,
            keep: is_kept,
        })
    }

    fn lookup(&mut self, wanted: DependencyLookup) -> Result<Lookup, Error> {
        let DependencyLookup { id, name } = wanted;
        let (manifest, lock, version) = match id {
            CrateId::Own(position) => {
                let own = &self.own[position];
                if self.extern_name(position).as_deref() == Some(name) {
                    return Ok(Lookup::Own(position));
                }
                let manifest = own.manifest.clone();
                let Some(lock) = self.lock_for(position)? else {
                    return Ok(Lookup::NoLockfile);
                };
                let is_listed = self.locks[lock].iter().any(|package| {
                    package.source.is_none() && Some(&package.name) == manifest.name.as_ref()
                });
                if !is_listed {
                    return Ok(Lookup::NoLockfile);
                }
                (manifest, lock, None)
            }
            CrateId::Package(package) => {
                let installed = &self.installed[package];
                (
                    installed.manifest.clone(),
                    installed.lock,
                    Some(installed.version.clone()),
                )
            }
        };
        let packages = &self.locks[lock];
        let entry = packages.iter().find(|package| {
            Some(&package.name) == manifest.name.as_ref()
                && version
                    .as_ref()
                    .map_or(package.source.is_none(), |version| {
                        *version == package.version
                    })
        });
        let declared = manifest
            .dependencies
            .iter()
            .find(|declared| declared.local == name)
            .or_else(|| {
                manifest.dependencies.iter().find(|declared| {
                    self.own.iter().enumerate().any(|(position, own)| {
                        own.manifest.name.as_ref() == Some(&declared.package)
                            && self.extern_name(position).as_deref() == Some(name)
                    })
                })
            });
        let locked = declared
            .zip(entry)
            .and_then(|(declared, entry)| {
                entry
                    .dependencies
                    .iter()
                    .find(|dependency| dependency.name == declared.package)
            })
            .and_then(|dependency| {
                packages.iter().find(|package| {
                    package.name == dependency.name
                        && dependency
                            .version
                            .as_ref()
                            .is_none_or(|version| *version == package.version)
                })
            });
        Ok(match locked {
            None => Lookup::Unknown,
            Some(package) if package.source.is_none() => self
                .own
                .iter()
                .position(|own| own.manifest.name.as_ref() == Some(&package.name))
                .map_or(Lookup::Unknown, Lookup::Own),
            Some(package) => Lookup::Locked {
                package: package.clone(),
                lock,
            },
        })
    }

    fn extern_name(&self, position: usize) -> Option<String> {
        let manifest = &self.own[position].manifest;
        manifest
            .library_name
            .as_ref()
            .or(manifest.name.as_ref())
            .map(|name| name.replace('-', "_"))
    }

    fn lock_for(&mut self, position: usize) -> Result<Option<usize>, Error> {
        let mut folder = self.own[position].folder.clone();
        loop {
            let path = folder.joined(LOCKFILE).unwrap_or_default();
            if self.tree.paths().contains(&path) {
                if let Some(index) = self.lock_of_file.get(&path) {
                    return Ok(Some(*index));
                }
                let text = self
                    .tree
                    .read(&[&path])?
                    .pop()
                    .map(|file| file.text)
                    .unwrap_or_default();
                self.locks.push(read_lockfile(&text)?);
                self.lock_paths.push(path.clone());
                self.lock_of_file.insert(path, self.locks.len() - 1);
                return Ok(Some(self.locks.len() - 1));
            }
            if folder.is_empty() {
                return Ok(None);
            }
            folder = parent_folder(&folder).to_string();
        }
    }
}
