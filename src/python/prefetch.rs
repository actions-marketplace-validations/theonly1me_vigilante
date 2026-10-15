use super::environment::Wanted;
use super::resolver::PythonResolver;
use super::standard_library::is_standard;
use crate::Error;
use crate::deep::program::{Import, Root};

impl PythonResolver<'_> {
    pub(super) fn prefetch_imports(&mut self, imports: &[Import]) -> Result<(), Error> {
        let mut wanted = Vec::new();
        for import in imports {
            let top = import.specifier.split('.').next().unwrap_or_default();
            if top.is_empty() || is_standard(top) {
                continue;
            }
            let project = match import.from.root {
                Root::Repository => self.environment.project_of(&import.from.path)?,
                Root::Package(package) => Some(self.environment.installed(package).project),
            };
            if let Some(project) = project {
                wanted.push((project, top.to_string()));
            }
        }
        wanted.sort();
        wanted.dedup();
        let wanted: Vec<Wanted> = wanted
            .iter()
            .map(|(project, module)| Wanted {
                project: *project,
                module,
            })
            .collect();
        self.environment.prefetch_modules(&wanted)
    }
}
