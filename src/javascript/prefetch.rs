use super::packages::Locate;
use super::parse::Parsed;
use super::paths::{is_built_in, is_relative, package_specifier};
use super::program::{FileKey, Program, Root};
use super::projects::Dependency;
use crate::Error;

struct Wanted {
    project: usize,
    folder: String,
    name: String,
    is_from_repository: bool,
    declared: Option<String>,
}

impl Program<'_> {
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
                    let is_from_repository = from.root == Root::Repository;
                    let declared = if is_from_repository {
                        self.input.packages.projects.declared(Dependency {
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
        self.input.packages.prefetch(&requests);
        Ok(())
    }
}
