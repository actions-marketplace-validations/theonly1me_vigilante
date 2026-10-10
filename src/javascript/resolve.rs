use std::collections::{BTreeSet, HashMap};

use super::manifest::Manifest;
use super::packages::{Locate, Located};
use super::parse::Parsed;
use super::paths::{is_built_in, is_relative, joined, package_specifier, parent_folder, probe};
use super::program::{FileKey, Program, Root};
use crate::Error;

const MANIFEST: &str = "package.json";
const NO_LOCKFILE: &str = "No package-lock.json, npm-shrinkwrap.json, or pnpm-lock.yaml covers this code, so Vigilante did not read its packages.";

struct Wanted {
    project: usize,
    folder: String,
    name: String,
    is_from_repository: bool,
}

impl Program<'_> {
    pub(super) fn resolve_all(
        &mut self,
        from: &FileKey,
        parsed: &mut Parsed,
    ) -> Result<HashMap<String, FileKey>, Error> {
        let specifiers: BTreeSet<String> = parsed
            .index
            .loads
            .iter()
            .map(|load| load.specifier.clone())
            .collect();
        let mut resolved = HashMap::new();
        for specifier in specifiers {
            let is_relative_code = is_relative(&specifier)
                && joined(parent_folder(&from.path), &specifier)
                    .and_then(|base| self.probe(&from.root, &base))
                    .is_some();
            if is_relative_code {
                parsed.privileged.remove(&specifier);
            }
            if parsed.privileged.contains_key(&specifier) {
                continue;
            }
            if let Some(target) = self.resolve(from, &specifier)? {
                resolved.insert(specifier, target);
            }
        }
        Ok(resolved)
    }

    pub(super) fn prefetch<'p>(
        &mut self,
        files: impl Iterator<Item = (&'p FileKey, &'p Parsed)>,
    ) -> Result<(), Error> {
        let mut wanted = Vec::new();
        for (from, parsed) in files {
            for load in &parsed.index.loads {
                let specifier = &load.specifier;
                if parsed.privileged.contains_key(specifier)
                    || is_built_in(specifier)
                    || is_relative(specifier)
                {
                    continue;
                }
                let Some(package) = package_specifier(specifier) else {
                    continue;
                };
                if let Some((project, folder)) = self.place_of(from)? {
                    wanted.push(Wanted {
                        project,
                        folder,
                        name: package.name.to_string(),
                        is_from_repository: from.root == Root::Repository,
                    });
                }
            }
        }
        let requests: Vec<Locate> = wanted
            .iter()
            .map(|item| Locate {
                project: item.project,
                folder: &item.folder,
                name: &item.name,
                is_from_repository: item.is_from_repository,
                is_declared: false,
            })
            .collect();
        self.input.packages.prefetch(&requests);
        Ok(())
    }

    fn resolve(&mut self, from: &FileKey, specifier: &str) -> Result<Option<FileKey>, Error> {
        if is_built_in(specifier) {
            return Ok(None);
        }
        if is_relative(specifier) {
            let Some(base) = joined(parent_folder(&from.path), specifier) else {
                return Ok(None);
            };
            return Ok(self.probe(&from.root, &base));
        }
        let Some(package) = package_specifier(specifier) else {
            return Ok(None);
        };
        let Some((project, folder)) = self.place_of(from)? else {
            let scope = (self.input.scope_of)(&from.path);
            self.input.packages.skip(&scope, NO_LOCKFILE.to_string());
            return Ok(None);
        };
        let is_from_repository = from.root == Root::Repository;
        let packages = &mut self.input.packages;
        let is_declared =
            is_from_repository && packages.projects.declares(&from.path, package.name)?;
        let located = packages.locate(Locate {
            project,
            folder: &folder,
            name: package.name,
            is_from_repository,
            is_declared,
        })?;
        let entries = match &located {
            Located::Missing => return Ok(None),
            Located::Package(index) => packages.installed(*index).manifest.entries(package.subpath),
            Located::Repository(folder) => {
                let manifest_path = joined(folder, MANIFEST).unwrap_or_default();
                let text = packages
                    .projects
                    .read_repository(&manifest_path)?
                    .unwrap_or_default();
                Manifest::parse(&text)
                    .entries(package.subpath)
                    .into_iter()
                    .filter_map(|entry| joined(folder, &entry))
                    .collect()
            }
        };
        let root = match located {
            Located::Package(index) => Root::Package(index),
            _ => Root::Repository,
        };
        Ok(entries.iter().find_map(|entry| self.probe(&root, entry)))
    }

    fn place_of(&mut self, from: &FileKey) -> Result<Option<(usize, String)>, Error> {
        let packages = &mut self.input.packages;
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

    fn probe(&self, root: &Root, base: &str) -> Option<FileKey> {
        let path = match root {
            Root::Repository => probe(
                &|path| self.input.packages.projects.has_repository_file(path),
                base,
            ),
            Root::Package(index) => {
                let files = &self.input.packages.installed(*index).files;
                probe(&|path| files.contains(path), base)
            }
        }?;
        Some(FileKey {
            root: root.clone(),
            path,
        })
    }
}
