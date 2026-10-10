use std::collections::BTreeSet;
use std::path::Path;

use super::install::Entry;
use super::plan::Planned;
use super::{Environment, Installed};
use crate::Error;
use crate::deep::Skip;
use crate::registry::{Fetched, folder_files};

const TOP_LEVEL: &str = "top_level.txt";
const METADATA_FOLDERS: &[&str] = &[".dist-info", ".egg-info", ".data"];
const NOT_MODULES: &[&str] = &[
    "setup",
    "conftest",
    "tests",
    "test",
    "docs",
    "examples",
    "benchmarks",
];

pub struct Fetching<'a> {
    pub planned: Planned,
    pub entry: Entry,
    pub lockfile: &'a str,
}

pub struct Layout {
    pub files: BTreeSet<String>,
    pub module_roots: Vec<String>,
    pub modules: BTreeSet<String>,
}

impl Environment<'_> {
    pub(super) fn fetch(&mut self, fetching: Fetching) -> Result<Option<usize>, Error> {
        let Fetching {
            planned,
            entry,
            lockfile,
        } = fetching;
        let label = format!("`{}=={}`", planned.name, planned.version);
        let folder = match planned.fetch_from(self.registry) {
            Ok(Fetched::Folder(folder)) => folder,
            Ok(Fetched::TooLarge) => {
                self.skip(Skip {
                    source: lockfile.to_string(),
                    reason: format!(
                        "{label} is larger than 256 MiB, so Vigilante did not read it."
                    ),
                });
                return Ok(None);
            }
            Ok(Fetched::NotFound) => {
                self.skip(Skip {
                    source: lockfile.to_string(),
                    reason: format!("{label} is not at `{}`.", planned.download.url),
                });
                return Ok(None);
            }
            Err(error) if planned.is_git => {
                self.skip(Skip {
                    source: lockfile.to_string(),
                    reason: format!("{label} could not be downloaded from GitHub: {error}"),
                });
                return Ok(None);
            }
            Err(error) => return Err(error),
        };
        let layout = layout(&folder)?;
        self.installed.push(Installed {
            name: planned.name,
            folder,
            files: layout.files,
            module_roots: layout.module_roots,
            modules: layout.modules,
            project: entry.project,
        });
        Ok(Some(self.installed.len() - 1))
    }
}

pub fn layout(folder: &Path) -> Result<Layout, Error> {
    let files = folder_files(folder)?;
    let mut modules: BTreeSet<String> = files
        .iter()
        .filter(|path| path.ends_with(&format!("/{TOP_LEVEL}")))
        .filter_map(|path| std::fs::read_to_string(folder.join(path)).ok())
        .flat_map(|text| {
            text.lines()
                .map(str::trim)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|module| !module.is_empty())
        .collect();
    let mut roots: BTreeSet<String> = BTreeSet::from([String::new()]);
    for path in &files {
        if let Some((top, _)) = path.split_once('/')
            && !METADATA_FOLDERS.iter().any(|suffix| top.ends_with(suffix))
        {
            roots.insert(top.to_string());
            roots.insert(format!("{top}/src"));
        }
    }
    let mut module_roots = Vec::new();
    for root in roots {
        let prefix = if root.is_empty() {
            String::new()
        } else {
            format!("{root}/")
        };
        let found: BTreeSet<String> = files
            .iter()
            .filter_map(|path| path.strip_prefix(&prefix))
            .filter_map(|rest| match rest.split_once('/') {
                Some((module, "__init__.py")) => Some(module.to_string()),
                None => rest.strip_suffix(".py").map(str::to_string),
                Some(_) => None,
            })
            .filter(|module| !NOT_MODULES.contains(&module.as_str()))
            .collect();
        if !found.is_empty() {
            modules.extend(found);
            module_roots.push(root);
        }
    }
    Ok(Layout {
        files,
        module_roots,
        modules,
    })
}
