use std::collections::HashMap;

use super::environment::{ClassLookup, JavaEnvironment};
use crate::Error;
use crate::deep::Skip;
use crate::deep::program::{FileKey, Import, PackageFile, Resolver, Root};
use crate::tree::Tree;

const PLATFORM_PACKAGES: &[&str] = &["java.", "javax.", "jdk.", "sun.", "com.sun.", "kotlin."];
const SOURCE_SUFFIX: &str = ".java";
const CLASS_SUFFIX: &str = ".class";

pub struct JavaResolver<'a> {
    pub environment: JavaEnvironment<'a>,
    own: HashMap<String, Vec<String>>,
}

pub struct ResolverInput<'a, 't> {
    pub environment: JavaEnvironment<'a>,
    pub tree: &'t dyn Tree,
}

impl<'a> JavaResolver<'a> {
    pub fn new(input: ResolverInput<'a, '_>) -> Self {
        let mut own: HashMap<String, Vec<String>> = HashMap::new();
        for path in input.tree.paths() {
            if let Some(name) = path
                .rsplit('/')
                .next()
                .filter(|name| name.ends_with(SOURCE_SUFFIX))
            {
                own.entry(name.to_string()).or_default().push(path.clone());
            }
        }
        Self {
            environment: input.environment,
            own,
        }
    }

    pub fn into_skipped(self) -> Vec<Skip> {
        self.environment.into_skipped()
    }

    fn own_file(&self, class: &str) -> Option<FileKey> {
        let parts: Vec<&str> = class.split('.').collect();
        (1..=parts.len()).rev().find_map(|length| {
            let top = parts[length - 1].split('$').next().unwrap_or_default();
            let folder = parts[..length - 1].join("/");
            let suffix = if folder.is_empty() {
                format!("{top}{SOURCE_SUFFIX}")
            } else {
                format!("{folder}/{top}{SOURCE_SUFFIX}")
            };
            self.own
                .get(&format!("{top}{SOURCE_SUFFIX}"))?
                .iter()
                .find(|path| *path == &suffix || path.ends_with(&format!("/{suffix}")))
                .map(|path| FileKey {
                    root: Root::Repository,
                    path: path.clone(),
                })
        })
    }
}

impl Resolver for JavaResolver<'_> {
    fn resolve(&mut self, import: Import) -> Result<Option<FileKey>, Error> {
        let class = import.specifier;
        if class.ends_with(".*")
            || PLATFORM_PACKAGES
                .iter()
                .any(|prefix| class.starts_with(prefix))
        {
            return Ok(None);
        }
        if let Some(found) = self.own_file(class) {
            return Ok(Some(found));
        }
        let project = match import.from.root {
            Root::Repository => self.environment.project_of(&import.from.path)?,
            Root::Package(archive) => Some(self.environment.archive(archive).project),
        };
        let Some(project) = project else {
            return Ok(None);
        };
        let parts: Vec<&str> = class.split('.').collect();
        Ok((1..=parts.len()).rev().find_map(|length| {
            let mut class_path = parts[..length].join("/");
            if length < parts.len() {
                class_path = format!("{class_path}${}", parts[length..].join("$"));
            }
            class_path.push_str(CLASS_SUFFIX);
            let archive = self.environment.class(ClassLookup {
                project,
                class_path: &class_path,
            })?;
            Some(FileKey {
                root: Root::Package(archive),
                path: class_path,
            })
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
        self.environment.archive(package).name.clone()
    }
}
