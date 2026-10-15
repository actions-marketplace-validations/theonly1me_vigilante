use super::manifest::Manifest;
use super::packages::{Locate, Located, Packages};
use super::paths::{Probe, is_built_in, is_relative, package_specifier, parent_folder, probe};
use super::projects::Dependency;
use crate::Error;
use crate::deep::Skip;
use crate::deep::program::{FileKey, Import, PackageFile, Resolver, Root};
use crate::repository_path::Folder;

const MANIFEST: &str = "package.json";
const NO_LOCKFILE: &str = "No package-lock.json, npm-shrinkwrap.json, pnpm-lock.yaml, or yarn.lock covers this code, so Vigilante did not read its packages.";

pub struct JavaScriptResolver<'a> {
    pub packages: Packages<'a>,
    pub scope_of: &'a dyn Fn(&str) -> String,
}

struct Candidate<'a> {
    root: &'a Root,
    base: &'a str,
}

impl Resolver for JavaScriptResolver<'_> {
    fn resolve(&mut self, import: Import) -> Result<Option<FileKey>, Error> {
        self.locate_import(import)
    }

    fn prefetch(&mut self, imports: &[Import]) -> Result<(), Error> {
        self.prefetch_imports(imports)
    }

    fn is_relative(&self, specifier: &str) -> bool {
        is_relative(specifier)
    }

    fn read(&self, file: PackageFile) -> Option<String> {
        self.packages.read(file)
    }

    fn package_name(&self, package: usize) -> String {
        self.packages.installed(package).name.clone()
    }
}

impl JavaScriptResolver<'_> {
    fn locate_import(&mut self, import: Import) -> Result<Option<FileKey>, Error> {
        let Import { from, specifier } = import;
        if is_built_in(specifier) {
            return Ok(None);
        }
        if is_relative(specifier) {
            let Some(base) = parent_folder(&from.path).joined(specifier) else {
                return Ok(None);
            };
            return Ok(self.probe(Candidate {
                root: &from.root,
                base: &base,
            }));
        }
        let Some(package) = package_specifier(specifier) else {
            return Ok(None);
        };
        let Some((project, folder)) = self.place_of(from)? else {
            let scope = (self.scope_of)(&from.path);
            self.packages.skip(Skip {
                source: scope,
                reason: NO_LOCKFILE.to_string(),
            });
            return Ok(None);
        };
        let is_from_repository = from.root == Root::Repository;
        let packages = &mut self.packages;
        let declared = if is_from_repository {
            packages.projects.declared(Dependency {
                path: &from.path,
                name: package.name,
            })?
        } else {
            None
        };
        let located = packages.locate(Locate {
            project,
            folder: &folder,
            name: package.name,
            is_from_repository,
            declared: declared.as_deref(),
        })?;
        let entries = match &located {
            Located::Missing => return Ok(None),
            Located::Package(index) => packages.installed(*index).manifest.entries(package.subpath),
            Located::Repository(folder) => {
                let manifest_path = folder.joined(MANIFEST).unwrap_or_default();
                let text = packages
                    .projects
                    .read_repository(&manifest_path)?
                    .unwrap_or_default();
                Manifest::parse(&text)
                    .entries(package.subpath)
                    .into_iter()
                    .filter_map(|entry| folder.joined(&entry))
                    .collect()
            }
        };
        let root = match located {
            Located::Package(index) => Root::Package(index),
            _ => Root::Repository,
        };
        Ok(entries.iter().find_map(|entry| {
            self.probe(Candidate {
                root: &root,
                base: entry,
            })
        }))
    }

    pub(super) fn place_of(&mut self, from: &FileKey) -> Result<Option<(usize, String)>, Error> {
        let packages = &mut self.packages;
        match &from.root {
            Root::Repository => {
                let Some(project) = packages.projects.project_of(&from.path)? else {
                    return Ok(None);
                };
                let project_folder = &packages.projects.project(project).folder;
                let folder = parent_folder(&from.path)
                    .strip_prefix(project_folder.as_str())
                    .unwrap_or_default()
                    .trim_start_matches('/')
                    .to_string();
                Ok(Some((project, folder)))
            }
            Root::Package(index) => {
                let installed = packages.installed(*index);
                Ok(Some((installed.project, installed.key.clone())))
            }
        }
    }

    fn probe(&self, candidate: Candidate) -> Option<FileKey> {
        let Candidate { root, base } = candidate;
        let path = match root {
            Root::Repository => probe(Probe {
                exists: &|path| self.packages.projects.has_repository_file(path),
                base,
            }),
            Root::Package(index) => {
                let files = &self.packages.installed(*index).files;
                probe(Probe {
                    exists: &|path| files.contains(path),
                    base,
                })
            }
        }?;
        Some(FileKey {
            root: root.clone(),
            path,
        })
    }
}
