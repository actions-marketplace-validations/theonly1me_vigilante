use std::collections::{BTreeSet, HashMap};

use super::parse::{ParseInput, Parsed, Source, parse_all};
use super::{
    FileKey, Import, Language, LoadedFile, PackageFile, Program, ProgramInput, Resolver, Root,
};
use crate::Error;
use crate::deep::graph::Owner;

struct ParsedFile<'a> {
    from: &'a FileKey,
    parsed: &'a mut Parsed,
}

impl<'a> Program<'a> {
    pub fn new(input: ProgramInput<'a>) -> Self {
        Self {
            input,
            files: Vec::new(),
            ids: HashMap::new(),
        }
    }

    pub fn language(&self) -> &dyn Language {
        self.input.language
    }

    pub fn resolver(&self) -> &dyn Resolver {
        &*self.input.resolver
    }

    pub fn load(&mut self, roots: &[String]) -> Result<(), Error> {
        let mut frontier: Vec<FileKey> = roots
            .iter()
            .map(|path| FileKey {
                root: Root::Repository,
                path: path.clone(),
            })
            .collect();
        while !frontier.is_empty() {
            frontier.retain(|key| !self.ids.contains_key(key));
            frontier.sort();
            frontier.dedup();
            let repository_paths: Vec<&str> = frontier
                .iter()
                .filter(|key| key.root == Root::Repository)
                .map(|key| key.path.as_str())
                .collect();
            let mut texts: HashMap<String, String> = self
                .input
                .tree
                .read(&repository_paths)?
                .into_iter()
                .map(|file| (file.path, file.text))
                .collect();
            let mut keys = Vec::new();
            let mut sources = Vec::new();
            for key in frontier {
                let text = match key.root {
                    Root::Repository => texts.remove(&key.path),
                    Root::Package(package) => self.input.resolver.read(PackageFile {
                        package,
                        path: &key.path,
                    }),
                };
                if let Some(text) = text {
                    let (display, owner) = self.identity(&key);
                    sources.push(Source { display, text });
                    keys.push((key, owner));
                }
            }
            let parsed = parse_all(ParseInput {
                language: self.input.language,
                rules: self.input.rules,
                sources: &sources,
            })?;
            self.prefetch(keys.iter().map(|(key, _)| key).zip(&parsed))?;
            let mut next = Vec::new();
            for (((key, owner), source), mut parsed) in keys.into_iter().zip(sources).zip(parsed) {
                let resolved = self.resolve_all(ParsedFile {
                    from: &key,
                    parsed: &mut parsed,
                })?;
                next.extend(resolved.values().cloned());
                self.ids.insert(key.clone(), self.files.len());
                self.files.push(LoadedFile {
                    key,
                    display: source.display,
                    owner,
                    index: parsed.index,
                    privileged: parsed.privileged,
                    sites: parsed.sites,
                    resolved,
                });
            }
            frontier = next;
        }
        Ok(())
    }

    fn identity(&self, key: &FileKey) -> (String, Owner) {
        match &key.root {
            Root::Repository => (
                key.path.clone(),
                Owner::Repository {
                    scope: (self.input.scope_of)(&key.path),
                },
            ),
            Root::Package(package) => {
                let name = self.input.resolver.package_name(*package);
                (
                    format!("{name}/{}", key.path),
                    Owner::Dependency { package: name },
                )
            }
        }
    }

    fn prefetch<'p>(
        &mut self,
        files: impl Iterator<Item = (&'p FileKey, &'p Parsed)>,
    ) -> Result<(), Error> {
        let mut imports = Vec::new();
        for (from, parsed) in files {
            for load in &parsed.index.loads {
                if !parsed.privileged.contains_key(&load.specifier) {
                    imports.push(Import {
                        from,
                        specifier: &load.specifier,
                    });
                }
            }
        }
        self.input.resolver.prefetch(&imports)
    }

    fn resolve_all(&mut self, file: ParsedFile) -> Result<HashMap<String, FileKey>, Error> {
        let ParsedFile { from, parsed } = file;
        let specifiers: BTreeSet<String> = parsed
            .index
            .loads
            .iter()
            .map(|load| load.specifier.clone())
            .collect();
        let mut resolved = HashMap::new();
        for specifier in specifiers {
            let is_privileged = parsed.privileged.contains_key(&specifier);
            if is_privileged && !self.input.resolver.is_relative(&specifier) {
                continue;
            }
            let target = self.input.resolver.resolve(Import {
                from,
                specifier: &specifier,
            })?;
            if let Some(target) = target {
                parsed.privileged.remove(&specifier);
                resolved.insert(specifier, target);
            }
        }
        Ok(resolved)
    }
}
