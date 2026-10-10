use super::manifest::Manifest;
use super::packages::{Locate, Located};
use super::paths::{is_built_in, is_relative, joined, package_specifier, parent_folder, probe};
use super::program::{FileKey, Program, Root};
use crate::Error;

const MANIFEST: &str = "package.json";

impl Program<'_> {
    pub(super) fn resolve(
        &mut self,
        from: &FileKey,
        specifier: &str,
    ) -> Result<Option<FileKey>, Error> {
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
        let packages = &mut self.input.packages;
        let (project, folder) = match &from.root {
            Root::Repository => {
                let Some(project) = packages.projects.project_of(&from.path)? else {
                    let scope = (self.input.scope_of)(&from.path);
                    packages.skip(&scope, "No package-lock.json or npm-shrinkwrap.json covers this code, so Vigilante did not read its packages.".to_string());
                    return Ok(None);
                };
                let project_folder = &packages.projects.project(project).folder;
                let folder = parent_folder(&from.path)
                    .strip_prefix(project_folder.as_str())
                    .unwrap_or_default()
                    .trim_start_matches('/')
                    .to_string();
                (project, folder)
            }
            Root::Package(index) => {
                let installed = packages.installed(*index);
                (installed.project, installed.key.clone())
            }
        };
        let located = packages.locate(Locate {
            project,
            folder: &folder,
            name: package.name,
            is_from_repository: from.root == Root::Repository,
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
