use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;
use tempfile::TempDir;

use crate::Error;
use crate::deep::program::{FileKey, PackageFile, Program, Root};
use crate::tree::Tree;

const PACKAGES: &str = "__vigilante_packages__";
const PYTHON: &str = ".py";
pub const CONFIG: &str = "pyrightconfig.json";
pub const ENVIRONMENT: &str = "environment.json";
pub const INDEX: &str = "index.scip";

pub struct ProjectInput<'a, 'p> {
    pub program: &'a Program<'p>,
    pub tree: &'a dyn Tree,
}

pub struct Project {
    pub _folder: TempDir,
    pub root: PathBuf,
    pub keys: HashMap<String, FileKey>,
}

pub fn write_project(input: ProjectInput) -> Result<Project, Error> {
    let ProjectInput { program, tree } = input;
    let folder = TempDir::new()
        .map_err(|error| Error::new(format!("could not create a temporary folder: {error}")))?;
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
    let mut own_texts: HashMap<String, String> = tree
        .read(&own_paths)?
        .into_iter()
        .map(|file| (file.path, file.text))
        .collect();
    let mut keys = HashMap::new();
    let mut search_roots = BTreeSet::new();
    for key in loaded {
        let (relative, text) = match key.root {
            Root::Repository => (key.path.clone(), own_texts.remove(&key.path)),
            Root::Package(package) => {
                for root in program.resolver().search_roots(package) {
                    search_roots.insert(joined(&format!("{PACKAGES}/{package}"), &root));
                }
                (
                    format!("{PACKAGES}/{package}/{}", key.path),
                    program.resolver().read(PackageFile {
                        package,
                        path: &key.path,
                    }),
                )
            }
        };
        let Some(text) = text else {
            continue;
        };
        write(&folder.path().join(&relative), text.as_bytes())?;
        keys.insert(relative, key.clone());
    }
    let config = json!({ "extraPaths": search_roots });
    write(&folder.path().join(CONFIG), config.to_string().as_bytes())?;
    write(&folder.path().join(ENVIRONMENT), b"[]")?;
    let root = fs::canonicalize(folder.path())
        .map_err(|error| Error::new(format!("could not read the temporary folder: {error}")))?;
    Ok(Project {
        _folder: folder,
        root,
        keys,
    })
}

fn joined(base: &str, root: &str) -> String {
    if root.is_empty() {
        base.to_string()
    } else {
        format!("{base}/{root}")
    }
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
