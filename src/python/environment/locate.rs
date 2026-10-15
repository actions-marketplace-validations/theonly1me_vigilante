use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use super::Environment;
use super::install::Entry;
use super::names::candidate_distributions;
use super::plan::{PYPI_JSON, Plan, PlanContext, plan};
use crate::Error;
use crate::lockfiles::python::PythonPackage;
use crate::registry::Prefetch;

const MAXIMUM_LOOKUPS_AT_ONCE: usize = 16;

pub struct Wanted<'a> {
    pub project: usize,
    pub module: &'a str,
}

pub enum Located {
    Package(usize),
    Unreadable,
    Missing,
}

impl Environment<'_> {
    pub fn locate(&mut self, wanted: Wanted) -> Result<Located, Error> {
        let Wanted { project, module } = wanted;
        if let Some(index) = self.projects[project].by_module.get(module).copied() {
            return self.located(Entry { project, index });
        }
        let candidates = self.candidates(&Wanted { project, module });
        let mut is_unreadable = false;
        for index in candidates {
            match self.install(Entry { project, index })? {
                Some(installed) if self.installed[installed].modules.contains(module) => {
                    self.projects[project]
                        .by_module
                        .insert(module.to_string(), index);
                    return Ok(Located::Package(installed));
                }
                Some(_) => {}
                None => is_unreadable = true,
            }
        }
        if is_unreadable {
            return Ok(Located::Unreadable);
        }
        if !self.projects[project].is_scanned {
            self.scan(project)?;
            if let Some(index) = self.projects[project].by_module.get(module).copied() {
                return self.located(Entry { project, index });
            }
        }
        Ok(Located::Missing)
    }

    fn located(&mut self, entry: Entry) -> Result<Located, Error> {
        Ok(match self.install(entry)? {
            Some(installed) => Located::Package(installed),
            None => Located::Unreadable,
        })
    }

    pub fn prefetch_modules(&mut self, wanted: &[Wanted]) -> Result<(), Error> {
        let mut entries = Vec::new();
        for item in wanted {
            entries.extend(self.candidates(item).into_iter().map(|index| Entry {
                project: item.project,
                index,
            }));
        }
        self.prefetch_entries(&entries)
    }

    fn candidates(&self, wanted: &Wanted) -> Vec<usize> {
        let packages = &self.projects[wanted.project].lock.packages;
        candidate_distributions(wanted.module)
            .iter()
            .filter_map(|name| packages.iter().position(|package| package.name == *name))
            .collect()
    }

    fn scan(&mut self, project: usize) -> Result<(), Error> {
        self.projects[project].is_scanned = true;
        let entries: Vec<Entry> = (0..self.projects[project].lock.packages.len())
            .map(|index| Entry { project, index })
            .collect();
        self.prefetch_entries(&entries)?;
        for entry in entries {
            if let Some(installed) = self.install(entry)? {
                for module in self.installed[installed].modules.clone() {
                    self.projects[project]
                        .by_module
                        .entry(module)
                        .or_insert(entry.index);
                }
            }
        }
        Ok(())
    }

    fn prefetch_entries(&self, entries: &[Entry]) -> Result<(), Error> {
        let packages: Vec<&PythonPackage> = entries
            .iter()
            .filter(|entry| {
                !self
                    .installed_at
                    .contains_key(&(entry.project, entry.index))
            })
            .map(|entry| &self.projects[entry.project].lock.packages[entry.index])
            .collect();
        let (registry, sources) = (self.registry, self.sources);
        let next = AtomicUsize::new(0);
        let workers = MAXIMUM_LOOKUPS_AT_ONCE.min(packages.len());
        let planned: Vec<Prefetch> = thread::scope(|scope| {
            let handles: Vec<_> = (0..workers)
                .map(|_| {
                    scope.spawn(|| {
                        let mut found = Vec::new();
                        while let Some(package) = packages.get(next.fetch_add(1, Ordering::Relaxed))
                        {
                            let result = plan(PlanContext {
                                registry,
                                sources,
                                index: PYPI_JSON,
                                package,
                            });
                            if let Ok(Plan::Download(item)) = result {
                                found.push(item.prefetch());
                            }
                        }
                        found
                    })
                })
                .collect();
            handles
                .into_iter()
                .flat_map(|handle| handle.join().unwrap_or_default())
                .collect()
        });
        self.registry.prefetch(&planned);
        Ok(())
    }
}
