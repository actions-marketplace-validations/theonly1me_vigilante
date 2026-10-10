use std::collections::{BTreeMap, BTreeSet};

use super::environment::{CrateId, CrateRoot, DependencyLookup, Located, RustEnvironment};
use super::modules::{ChildModule, ModuleTree};
use crate::Error;
use crate::deep::Skip;
use crate::deep::program::{FileKey, Import, PackageFile, Resolver, Root};

const STANDARD_CRATES: &[&str] = &["std", "core", "alloc", "proc_macro", "test"];
const RELATIVE: &[&str] = &["crate", "self", "super"];
const SEPARATOR: &str = "::";
const NO_LOCKFILE: &str = "No Cargo.lock covers this code, so Vigilante did not read its crates.";

pub struct RustResolver<'a> {
    pub environment: RustEnvironment<'a>,
    pub scope_of: &'a dyn Fn(&str) -> String,
    pub unresolved: BTreeMap<String, BTreeSet<String>>,
}

struct External<'a> {
    from: &'a FileKey,
    id: CrateId,
    name: &'a str,
}

impl<'a> RustResolver<'a> {
    fn child(&self, wanted: ChildModule) -> Option<FileKey> {
        let root = self.environment.crate_root(wanted.file)?;
        self.tree(&root).child(wanted)
    }

    fn tree<'t>(&'t self, root: &'t CrateRoot) -> ModuleTree<'t, 'a> {
        ModuleTree {
            environment: &self.environment,
            root_file: &root.file,
        }
    }

    fn external(&mut self, wanted: External) -> Result<Option<FileKey>, Error> {
        let External { from, id, name } = wanted;
        let scope = || (self.scope_of)(&from.path);
        Ok(
            match self.environment.dependency(DependencyLookup { id, name })? {
                Located::Own(position) => self.environment.library_root(CrateId::Own(position)),
                Located::Package(package) => {
                    self.environment.library_root(CrateId::Package(package))
                }
                Located::Unknown => {
                    if from.root == Root::Repository && name.starts_with(char::is_lowercase) {
                        let scope = scope();
                        self.unresolved
                            .entry(scope)
                            .or_default()
                            .insert(name.to_string());
                    }
                    None
                }
                Located::NoLockfile => {
                    if from.root == Root::Repository {
                        let source = scope();
                        self.environment.skip(Skip {
                            source,
                            reason: NO_LOCKFILE.to_string(),
                        });
                    }
                    None
                }
                Located::Skipped => None,
            },
        )
    }

    fn start(&mut self, import: &Import) -> Result<Option<(FileKey, usize)>, Error> {
        let parts: Vec<&str> = import.specifier.split(SEPARATOR).collect();
        let Some(context) = self.environment.crate_root(import.from) else {
            return Ok(None);
        };
        Ok(match parts[0] {
            "crate" => Some((context.file, 1)),
            "self" => Some((import.from.clone(), 1)),
            "super" => {
                let count = parts.iter().take_while(|part| **part == "super").count();
                let mut file = Some(import.from.clone());
                for _ in 0..count {
                    file = file.and_then(|file| self.tree(&context).parent(&file));
                }
                file.map(|file| (file, count))
            }
            name => match self.child(ChildModule {
                file: import.from,
                name,
            }) {
                Some(child) => Some((child, 1)),
                None => self
                    .external(External {
                        from: import.from,
                        id: context.id,
                        name,
                    })?
                    .map(|root| (root, 1)),
            },
        })
    }

    pub fn into_skipped(self) -> Vec<Skip> {
        let mut skipped = self.environment.into_skipped();
        for (scope, crates) in self.unresolved {
            let names: Vec<String> = crates
                .iter()
                .take(5)
                .map(|name| format!("`{name}`"))
                .collect();
            skipped.push(Skip {
                source: scope,
                reason: format!(
                    "Vigilante found {} neither in the repository nor in Cargo.lock, so it did not read {}.",
                    names.join(", "),
                    if crates.len() == 1 { "it" } else { "them" }
                ),
            });
        }
        skipped
    }
}

impl Resolver for RustResolver<'_> {
    fn resolve(&mut self, import: Import) -> Result<Option<FileKey>, Error> {
        let parts: Vec<&str> = import.specifier.split(SEPARATOR).collect();
        if parts.iter().any(|part| part.is_empty()) || STANDARD_CRATES.contains(&parts[0]) {
            return Ok(None);
        }
        let Some((mut file, used)) = self.start(&import)? else {
            return Ok(None);
        };
        for name in &parts[used..] {
            match self.child(ChildModule { file: &file, name }) {
                Some(child) => file = child,
                None => break,
            }
        }
        Ok(Some(file))
    }

    fn prefetch(&mut self, imports: &[Import]) -> Result<(), Error> {
        let mut planned = Vec::new();
        for import in imports {
            let first = import.specifier.split(SEPARATOR).next().unwrap_or_default();
            if RELATIVE.contains(&first) || STANDARD_CRATES.contains(&first) {
                continue;
            }
            let Some(context) = self.environment.crate_root(import.from) else {
                continue;
            };
            if let Some(item) = self.environment.planned(DependencyLookup {
                id: context.id,
                name: first,
            }) && !planned.iter().any(|known: &crate::registry::Prefetch| {
                known.name == item.name && known.version == item.version
            }) {
                planned.push(item);
            }
        }
        self.environment.prefetch(&planned);
        Ok(())
    }

    fn is_relative(&self, specifier: &str) -> bool {
        RELATIVE.contains(&specifier.split(SEPARATOR).next().unwrap_or_default())
    }

    fn read(&self, file: PackageFile) -> Option<String> {
        self.environment.read(file)
    }

    fn package_name(&self, package: usize) -> String {
        self.environment.installed(package).name.clone()
    }
}
