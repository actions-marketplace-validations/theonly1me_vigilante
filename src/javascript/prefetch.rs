use super::packages::Locate;
use super::paths::{is_built_in, is_relative, package_specifier};
use super::projects::Dependency;
use super::resolver::JavaScriptResolver;
use crate::Error;
use crate::deep::program::{Import, Root};

struct Wanted {
    project: usize,
    folder: String,
    name: String,
    is_from_repository: bool,
    declared: Option<String>,
}

impl JavaScriptResolver<'_> {
    pub(super) fn prefetch_imports(&mut self, imports: &[Import]) -> Result<(), Error> {
        let mut wanted = Vec::new();
        for import in imports {
            let (from, specifier) = (import.from, import.specifier);
            {
                if is_built_in(specifier) || is_relative(specifier) {
                    continue;
                }
                let Some(package) = package_specifier(specifier) else {
                    continue;
                };
                if let Some((project, folder)) = self.place_of(from)? {
                    let is_from_repository = from.root == Root::Repository;
                    let declared = if is_from_repository {
                        self.packages.projects.declared(Dependency {
                            path: &from.path,
                            name: package.name,
                        })?
                    } else {
                        None
                    };
                    wanted.push(Wanted {
                        project,
                        folder,
                        name: package.name.to_string(),
                        is_from_repository,
                        declared,
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
                declared: item.declared.as_deref(),
            })
            .collect();
        self.packages.prefetch(&requests);
        Ok(())
    }
}
