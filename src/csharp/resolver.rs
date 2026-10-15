use super::environment::{CSharpEnvironment, TypeQuery};
use super::syntax::CANDIDATE_MARK;
use crate::Error;
use crate::deep::Skip;
use crate::deep::program::{FileKey, Import, PackageFile, Resolver, Root};

const CANDIDATE_SEPARATOR: char = ',';

pub struct CSharpResolver<'a> {
    pub environment: CSharpEnvironment<'a>,
}

impl CSharpResolver<'_> {
    pub fn into_skipped(self) -> Vec<Skip> {
        self.environment.into_skipped()
    }
}

impl Resolver for CSharpResolver<'_> {
    fn resolve(&mut self, import: Import) -> Result<Option<FileKey>, Error> {
        let Some((name, candidates)) = import.specifier.split_once(CANDIDATE_MARK) else {
            return Ok(None);
        };
        let project = match import.from.root {
            Root::Repository => self.environment.project_of(&import.from.path)?,
            Root::Package(package) => Some(self.environment.package_project(package)),
        };
        Ok(candidates.split(CANDIDATE_SEPARATOR).find_map(|candidate| {
            let key = if candidate.is_empty() {
                name.to_string()
            } else {
                format!("{candidate}.{name}")
            };
            self.environment.type_file(TypeQuery { project, key: &key })
        }))
    }

    fn prefetch(&mut self, _imports: &[Import]) -> Result<(), Error> {
        Ok(())
    }

    fn is_relative(&self, _specifier: &str) -> bool {
        false
    }

    fn read(&self, file: PackageFile) -> Option<String> {
        self.environment.read(file)
    }

    fn package_name(&self, package: usize) -> String {
        self.environment.package_name(package)
    }
}
