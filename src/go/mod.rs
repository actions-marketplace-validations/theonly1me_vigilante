mod capslock;
mod checkout;
mod mapping;

use std::collections::BTreeSet;
use std::path::Path;

use crate::config::Config;
use crate::scan::Location;
use crate::tree::Tree;
use crate::{Error, Scope, Sighting, Surface};
use checkout::{Checkout, CheckoutRequest};
use mapping::Module;

const MODULE_FILE: &str = "go.mod";

pub struct GoModules {
    modules: Vec<Module>,
}

pub struct DiscoverInput<'a> {
    pub tree: &'a dyn Tree,
    pub config: &'a Config,
}

pub struct GoRequest<'a> {
    pub changed: Option<&'a BTreeSet<String>>,
    pub location: Location<'a>,
    pub config: &'a Config,
}

impl GoModules {
    pub fn discover(input: DiscoverInput) -> Result<Self, Error> {
        let module_files: Vec<&str> = input
            .tree
            .paths()
            .iter()
            .map(String::as_str)
            .filter(|path| {
                path.rsplit('/').next() == Some(MODULE_FILE) && !input.config.is_excluded(path)
            })
            .collect();
        let modules = input
            .tree
            .read(&module_files)?
            .into_iter()
            .map(|file| {
                let path = module_path(&file.text)
                    .ok_or_else(|| Error::new(format!("{} has no module line", file.path)))?;
                let folder = file
                    .path
                    .rsplit_once('/')
                    .map_or(".", |(folder, _)| folder)
                    .to_string();
                Ok(Module { folder, path })
            })
            .collect::<Result<_, Error>>()?;
        Ok(Self { modules })
    }

    pub fn owns(&self, path: &str) -> bool {
        path.ends_with(".go") && self.containing(path).is_some()
    }

    pub fn analyze(&self, request: GoRequest) -> Result<Vec<Sighting>, Error> {
        let selected: Vec<&Module> = match request.changed {
            None => self.modules.iter().collect(),
            Some(changed) => self
                .modules
                .iter()
                .filter(|module| {
                    changed.iter().any(|path| {
                        self.containing(path)
                            .is_some_and(|owner| owner.folder == module.folder)
                    })
                })
                .collect(),
        };
        if selected.is_empty() {
            return Ok(Vec::new());
        }
        capslock::ensure_tools()?;
        let checkout;
        let root: &Path = match request.location {
            Location::WorkingDirectory(repository) => repository,
            Location::Commit { repository, commit } => {
                checkout = Checkout::new(CheckoutRequest { repository, commit })?;
                checkout.root()
            }
        };
        let mut sightings = Vec::new();
        for module in selected {
            for info in capslock::analyze(&root.join(&module.folder))? {
                let power = mapping::power_of(&info.capability_name)?;
                let Some(folder) = mapping::folder_of(&info.package_dir, &self.modules) else {
                    continue;
                };
                if !self.is_owned_by(OwnedFolder {
                    folder: &folder,
                    module,
                }) {
                    continue;
                }
                let evidence = mapping::evidence_of(&info, &folder);
                if request.config.is_excluded(&evidence.path) {
                    continue;
                }
                let path = request
                    .config
                    .named_scope_of(&evidence.path)
                    .map_or(folder, str::to_string);
                sightings.push(Sighting {
                    scope: Scope {
                        surface: Surface::Code,
                        path,
                    },
                    power,
                    evidence,
                });
            }
        }
        Ok(sightings)
    }

    fn is_owned_by(&self, owned: OwnedFolder) -> bool {
        self.containing(&mapping::joined(owned.folder, "package.go"))
            .is_some_and(|owner| owner.folder == owned.module.folder)
    }

    fn containing(&self, path: &str) -> Option<&Module> {
        self.modules
            .iter()
            .filter(|module| {
                module.folder == "."
                    || path
                        .strip_prefix(module.folder.as_str())
                        .is_some_and(|rest| rest.starts_with('/'))
            })
            .max_by_key(|module| module.folder.len())
    }
}

struct OwnedFolder<'a> {
    folder: &'a str,
    module: &'a Module,
}

fn module_path(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("module"))
        .filter(|rest| rest.starts_with([' ', '\t']))
        .map(|rest| {
            let without_comment = rest.split("//").next().unwrap_or_default();
            without_comment.trim().trim_matches(['"', '`']).to_string()
        })
        .filter(|path| !path.is_empty())
}
