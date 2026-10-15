use std::collections::{BTreeMap, BTreeSet};

use super::environment::{Environment, Located, Wanted};
use super::standard_library::is_standard;
use crate::Error;
use crate::deep::Skip;
use crate::deep::program::{FileKey, Import, PackageFile, Resolver, Root};
use crate::repository_path::{Folder, parent_folder};
use crate::tree::Tree;

const NO_LOCKFILE: &str = "No uv.lock, poetry.lock, Pipfile.lock, or requirements.txt covers this code, so Vigilante did not read its packages.";

pub struct PythonResolver<'a> {
    pub environment: Environment<'a>,
    pub tree: &'a dyn Tree,
    pub scope_of: &'a dyn Fn(&str) -> String,
    pub unresolved: BTreeMap<String, BTreeSet<String>>,
}

struct InRoot<'a> {
    root: &'a Root,
    module_path: &'a str,
}

impl Resolver for PythonResolver<'_> {
    fn resolve(&mut self, import: Import) -> Result<Option<FileKey>, Error> {
        let Import { from, specifier } = import;
        if specifier.starts_with('.') {
            return Ok(self.relative(import));
        }
        let module_path = specifier.replace('.', "/");
        if from.root == Root::Repository {
            for root in own_roots(OwnFile {
                path: &from.path,
                scope: &(self.scope_of)(&from.path),
            }) {
                if let Some(found) = self.probe(InRoot {
                    root: &Root::Repository,
                    module_path: &root.joined(&module_path).unwrap_or_default(),
                }) {
                    return Ok(Some(found));
                }
            }
        }
        let top = specifier.split('.').next().unwrap_or(specifier);
        if let Root::Package(package) = from.root
            && self.environment.installed(package).modules.contains(top)
        {
            return Ok(self.in_package(InPackage {
                package,
                module_path: &module_path,
            }));
        }
        if is_standard(specifier) {
            return Ok(None);
        }
        let project = match from.root {
            Root::Repository => self.environment.project_of(&from.path)?,
            Root::Package(package) => Some(self.environment.installed(package).project),
        };
        let Some(project) = project else {
            if from.root == Root::Repository {
                self.environment.skip(Skip {
                    source: (self.scope_of)(&from.path),
                    reason: NO_LOCKFILE.to_string(),
                });
            }
            return Ok(None);
        };
        match self.environment.locate(Wanted {
            project,
            module: top,
        })? {
            Located::Package(package) => Ok(self.in_package(InPackage {
                package,
                module_path: &module_path,
            })),
            Located::Unreadable => Ok(None),
            Located::Missing => {
                if from.root == Root::Repository && !specifier.contains('.') {
                    self.unresolved
                        .entry((self.scope_of)(&from.path))
                        .or_default()
                        .insert(top.to_string());
                }
                Ok(None)
            }
        }
    }

    fn prefetch(&mut self, imports: &[Import]) -> Result<(), Error> {
        self.prefetch_imports(imports)
    }

    fn is_relative(&self, specifier: &str) -> bool {
        specifier.starts_with('.')
    }

    fn read(&self, file: PackageFile) -> Option<String> {
        self.environment.read(file)
    }

    fn package_name(&self, package: usize) -> String {
        self.environment.installed(package).name.clone()
    }
}

struct InPackage<'a> {
    package: usize,
    module_path: &'a str,
}

struct OwnFile<'a> {
    path: &'a str,
    scope: &'a str,
}

impl PythonResolver<'_> {
    fn relative(&self, import: Import) -> Option<FileKey> {
        let dots = import
            .specifier
            .chars()
            .take_while(|character| *character == '.')
            .count();
        let rest = &import.specifier[dots..];
        let mut package = parent_folder(&import.from.path).to_string();
        for _ in 1..dots {
            package = parent_folder(&package).to_string();
        }
        self.probe(InRoot {
            root: &import.from.root,
            module_path: &package.joined(&rest.replace('.', "/")).unwrap_or_default(),
        })
    }

    fn in_package(&self, wanted: InPackage) -> Option<FileKey> {
        let installed = self.environment.installed(wanted.package);
        installed.module_roots.iter().find_map(|root| {
            self.probe(InRoot {
                root: &Root::Package(wanted.package),
                module_path: &root.joined(wanted.module_path).unwrap_or_default(),
            })
        })
    }

    fn probe(&self, wanted: InRoot) -> Option<FileKey> {
        let exists = |path: &str| match wanted.root {
            Root::Repository => self.tree.paths().contains(path),
            Root::Package(package) => self.environment.installed(*package).files.contains(path),
        };
        let module = wanted.module_path.trim_end_matches('/');
        [
            format!("{module}.py"),
            module.joined("__init__.py").unwrap_or_default(),
        ]
        .into_iter()
        .find(|candidate| !candidate.is_empty() && exists(candidate))
        .map(|path| FileKey {
            root: wanted.root.clone(),
            path,
        })
    }

    pub fn into_skipped(self) -> Vec<Skip> {
        let mut skipped = self.environment.into_skipped();
        for (scope, modules) in self.unresolved {
            let names: Vec<String> = modules
                .iter()
                .take(5)
                .map(|module| format!("`{module}`"))
                .collect();
            skipped.push(Skip {
                source: scope,
                reason: format!(
                    "Vigilante found {} neither in the repository nor in the lockfile, so it did not read {}.",
                    names.join(", "),
                    if modules.len() == 1 { "it" } else { "them" }
                ),
            });
        }
        skipped
    }
}

fn own_roots(file: OwnFile) -> Vec<String> {
    let scope = if file.scope == "." { "" } else { file.scope };
    let mut roots = vec![
        parent_folder(file.path).to_string(),
        scope.to_string(),
        scope.joined("src").unwrap_or_default(),
        String::new(),
    ];
    roots.dedup();
    roots
}
