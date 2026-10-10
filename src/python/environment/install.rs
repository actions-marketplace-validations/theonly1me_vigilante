use super::Environment;
use super::layout::Fetching;
use super::plan::{PYPI_JSON, Plan, PlanContext, plan};
use crate::Error;
use crate::deep::Skip;

#[derive(Clone, Copy)]
pub struct Entry {
    pub project: usize,
    pub index: usize,
}

impl Environment<'_> {
    pub fn install(&mut self, entry: Entry) -> Result<Option<usize>, Error> {
        if let Some(known) = self.installed_at.get(&(entry.project, entry.index)) {
            return Ok(*known);
        }
        let project = &self.projects[entry.project];
        let lockfile = project.file.clone();
        let result = plan(PlanContext {
            registry: self.registry,
            sources: self.sources,
            index: PYPI_JSON,
            package: &project.lock.packages[entry.index],
        })?;
        let installed = match result {
            Plan::Download(planned) => self.fetch(Fetching {
                planned,
                entry,
                lockfile: &lockfile,
            })?,
            Plan::Skip(reason) => {
                self.skip(Skip {
                    source: lockfile,
                    reason,
                });
                None
            }
            Plan::Local => None,
        };
        self.installed_at
            .insert((entry.project, entry.index), installed);
        Ok(installed)
    }
}
