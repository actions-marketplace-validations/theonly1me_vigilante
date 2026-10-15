use std::collections::BTreeSet;
use std::iter;

use crate::config::Config;

const ROOT: &str = ".";

const MANIFEST_NAMES: &[&str] = &[
    "Cargo.toml",
    "package.json",
    "pyproject.toml",
    "setup.py",
    "setup.cfg",
    "go.mod",
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    "Gemfile",
];

const MANIFEST_EXTENSIONS: &[&str] = &[".gemspec", ".csproj"];

pub struct ScopesInput<'a> {
    pub paths: &'a BTreeSet<String>,
    pub config: &'a Config,
}

pub struct Scopes<'a> {
    config: &'a Config,
    package_roots: BTreeSet<&'a str>,
}

impl<'a> Scopes<'a> {
    pub fn new(input: ScopesInput<'a>) -> Self {
        let package_roots = input
            .paths
            .iter()
            .filter(|path| is_manifest(path) && !input.config.is_excluded(path))
            .filter_map(|path| parent_folders(path).next())
            .collect();
        Self {
            config: input.config,
            package_roots,
        }
    }

    pub fn code_scope_of(&self, path: &str) -> String {
        if let Some(name) = self.config.named_scope_of(path) {
            return name.to_string();
        }
        parent_folders(path)
            .find(|folder| self.package_roots.contains(folder))
            .unwrap_or_else(|| top_folder(path))
            .to_string()
    }
}

fn is_manifest(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    MANIFEST_NAMES.contains(&name)
        || MANIFEST_EXTENSIONS
            .iter()
            .any(|extension| name.ends_with(extension))
}

fn parent_folders(path: &str) -> impl Iterator<Item = &str> {
    let mut remaining = Some(path);
    iter::from_fn(move || {
        let current = remaining?;
        match current.rsplit_once('/') {
            Some((parent, _)) => {
                remaining = Some(parent);
                Some(parent)
            }
            None => {
                remaining = None;
                Some(ROOT)
            }
        }
    })
}

fn top_folder(path: &str) -> &str {
    path.split_once('/').map_or(ROOT, |(top, _)| top)
}
