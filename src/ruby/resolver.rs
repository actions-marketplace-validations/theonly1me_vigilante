use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::environment::RubyEnvironment;
use super::names::{constant_path, gem_hints, underscore};
use super::search::{Lookup, Searched};
use super::standard_library::is_standard;
use super::syntax::{CANDIDATE_MARK, NAMESPACE_SEPARATOR};
use crate::Error;
use crate::deep::Skip;
use crate::deep::program::{FileKey, Import, PackageFile, Resolver, Root};
use crate::repository_path::{Folder, parent_folder};

const SOURCE_SUFFIX: &str = ".rb";
const NO_LOCKFILE: &str = "No Gemfile.lock covers this code, so Vigilante did not read its gems.";

pub struct RubyResolver<'a> {
    pub environment: RubyEnvironment<'a>,
    pub scope_of: &'a dyn Fn(&str) -> String,
    pub unresolved: BTreeMap<String, BTreeSet<String>>,
    pub own_by_name: HashMap<String, Vec<String>>,
}

impl RubyResolver<'_> {
    pub fn into_skipped(self) -> Vec<Skip> {
        let mut skipped = self.environment.into_skipped();
        for (scope, names) in self.unresolved {
            let listed: Vec<String> = names
                .iter()
                .take(5)
                .map(|name| format!("`{name}`"))
                .collect();
            skipped.push(Skip {
                source: scope,
                reason: format!(
                    "Vigilante found {} neither in the repository nor in Gemfile.lock, so it did not read {}.",
                    listed.join(", "),
                    if names.len() == 1 { "it" } else { "them" }
                ),
            });
        }
        skipped
    }
}

impl Resolver for RubyResolver<'_> {
    fn resolve(&mut self, import: Import) -> Result<Option<FileKey>, Error> {
        let Import { from, specifier } = import;
        if let Some((name, candidates)) = specifier.split_once(CANDIDATE_MARK) {
            for candidate in candidates.split(',') {
                let full = if candidate.is_empty() {
                    name.to_string()
                } else {
                    format!("{candidate}{NAMESPACE_SEPARATOR}{name}")
                };
                let first = full.split(NAMESPACE_SEPARATOR).next().unwrap_or_default();
                if let Searched::Found(found) = self.find(Lookup {
                    from,
                    relative: constant_path(&full),
                    hints: gem_hints(&underscore(first)),
                    scan_all: false,
                })? {
                    return Ok(Some(found));
                }
            }
            return Ok(None);
        }
        let relative = if specifier.ends_with(SOURCE_SUFFIX) {
            specifier.to_string()
        } else {
            format!("{specifier}{SOURCE_SUFFIX}")
        };
        if specifier.starts_with('.') {
            return Ok(parent_folder(&from.path)
                .joined(&relative)
                .map(|path| FileKey {
                    root: from.root.clone(),
                    path,
                })
                .filter(|file| self.environment.exists(file)));
        }
        let first = specifier.split('/').next().unwrap_or(specifier).to_string();
        let mut hints = vec![specifier.replace('/', "-")];
        hints.extend(gem_hints(&first));
        hints.dedup();
        let searched = self.find(Lookup {
            from,
            relative,
            hints,
            scan_all: true,
        })?;
        if let Searched::Found(found) = searched {
            return Ok(Some(found));
        }
        if matches!(searched, Searched::Missing)
            && from.root == Root::Repository
            && !is_standard(specifier)
        {
            let scope = (self.scope_of)(&from.path);
            if self.environment.lock_for(&from.path)?.is_none() {
                self.environment.skip(Skip {
                    source: scope,
                    reason: NO_LOCKFILE.to_string(),
                });
            } else {
                self.unresolved.entry(scope).or_default().insert(first);
            }
        }
        Ok(None)
    }

    fn prefetch(&mut self, _imports: &[Import]) -> Result<(), Error> {
        Ok(())
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
