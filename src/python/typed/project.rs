use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;
use tempfile::TempDir;

use crate::Error;
use crate::deep::program::{FileKey, PackageFile, Program, Root};
use crate::tree::Tree;

const PYTHON: &str = ".py";
pub const PACKAGE_PREFIX: &str = "vigilante-package-";
pub const CONFIG: &str = "pyrightconfig.json";
pub const ENVIRONMENT: &str = "environment.json";
pub const INDEX: &str = "index.scip";

pub struct ProjectInput<'a, 'p> {
    pub program: &'a Program<'p>,
    pub tree: &'a dyn Tree,
}

pub struct Project {
    pub _own: TempDir,
    pub _packages: TempDir,
    pub root: PathBuf,
    pub keys: HashMap<String, FileKey>,
    pub modules: HashMap<(usize, String), FileKey>,
}

pub fn write_project(input: ProjectInput) -> Result<Project, Error> {
    let ProjectInput { program, tree } = input;
    let own = temporary()?;
    let packages = temporary()?;
    let packages_root = canonical(packages.path())?;
    let loaded: Vec<&FileKey> = program
        .files
        .iter()
        .map(|file| &file.key)
        .filter(|key| key.path.ends_with(PYTHON))
        .collect();
    let own_paths: Vec<&str> = loaded
        .iter()
        .filter(|key| key.root == Root::Repository)
        .map(|key| key.path.as_str())
        .collect();
    let mut keys = HashMap::new();
    for file in tree.read(&own_paths)? {
        write(&own.path().join(&file.path), file.text.as_bytes())?;
        keys.insert(
            file.path.clone(),
            FileKey {
                root: Root::Repository,
                path: file.path,
            },
        );
    }
    let mut modules = HashMap::new();
    let mut environment: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    let mut search_roots = Vec::new();
    for key in loaded {
        let Root::Package(package) = key.root else {
            continue;
        };
        let Some(text) = program.resolver().read(PackageFile {
            package,
            path: &key.path,
        }) else {
            continue;
        };
        let folder = packages_root.join(package.to_string());
        write(&folder.join(&key.path), text.as_bytes())?;
        for root in program.resolver().search_roots(package) {
            let Some(relative) = within(&key.path, &root) else {
                continue;
            };
            let search_root = folder.join(&root);
            if !search_roots.contains(&search_root) {
                search_roots.push(search_root);
            }
            environment
                .entry(package)
                .or_default()
                .push(relative.clone());
            modules.insert((package, relative), key.clone());
        }
    }
    let config = json!({ "extraPaths": search_roots });
    write(&own.path().join(CONFIG), config.to_string().as_bytes())?;
    let entries: Vec<_> = environment
        .into_iter()
        .map(|(package, files)| {
            json!({ "name": format!("{PACKAGE_PREFIX}{package}"), "version": "0", "files": files })
        })
        .collect();
    write(
        &own.path().join(ENVIRONMENT),
        json!(entries).to_string().as_bytes(),
    )?;
    Ok(Project {
        root: canonical(own.path())?,
        _own: own,
        _packages: packages,
        keys,
        modules,
    })
}

fn within(path: &str, root: &str) -> Option<String> {
    if root.is_empty() {
        Some(path.to_string())
    } else {
        path.strip_prefix(&format!("{root}/")).map(str::to_string)
    }
}

fn temporary() -> Result<TempDir, Error> {
    TempDir::new()
        .map_err(|error| Error::new(format!("could not create a temporary folder: {error}")))
}

fn canonical(path: &Path) -> Result<PathBuf, Error> {
    fs::canonicalize(path)
        .map_err(|error| Error::new(format!("could not read a temporary folder: {error}")))
}

fn write(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            Error::new(format!("could not create {}: {error}", parent.display()))
        })?;
    }
    fs::write(path, bytes)
        .map_err(|error| Error::new(format!("could not write {}: {error}", path.display())))
}
