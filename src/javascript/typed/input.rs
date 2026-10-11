use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use serde::Serialize;

use crate::Error;
use crate::deep::program::{FileKey, PackageFile, Program, Root};
use crate::javascript::paths::is_code_file;
use crate::tree::Tree;

const REPOSITORY: &str = "/vigilante/repository/";
const PACKAGE: &str = "/vigilante/package-";

pub struct CheckerInput<'a, 'p> {
    pub program: &'a Program<'p>,
    pub tree: &'a dyn Tree,
    pub compiler: &'a Path,
}

#[derive(Default)]
pub struct VirtualFiles {
    keys: HashMap<String, FileKey>,
}

impl VirtualFiles {
    pub fn key_of(&self, name: &str) -> Option<FileKey> {
        self.keys.get(name).cloned()
    }
}

#[derive(Serialize)]
struct Input<'a> {
    typescript: &'a Path,
    files: Vec<InputFile>,
}

#[derive(Serialize)]
struct InputFile {
    name: String,
    text: String,
    imports: BTreeMap<String, String>,
}

pub fn checker_input(input: CheckerInput) -> Result<(Vec<u8>, VirtualFiles), Error> {
    let CheckerInput {
        program,
        tree,
        compiler,
    } = input;
    let loaded: Vec<_> = program
        .files
        .iter()
        .filter(|file| is_code_file(&file.key.path))
        .collect();
    let own_paths: Vec<&str> = loaded
        .iter()
        .filter(|file| file.key.root == Root::Repository)
        .map(|file| file.key.path.as_str())
        .collect();
    let mut own_texts: HashMap<String, String> = tree
        .read(&own_paths)?
        .into_iter()
        .map(|file| (file.path, file.text))
        .collect();
    let mut files = VirtualFiles::default();
    let mut entries = Vec::new();
    for file in loaded {
        let text = match file.key.root {
            Root::Repository => own_texts.remove(&file.key.path),
            Root::Package(package) => program.resolver().read(PackageFile {
                package,
                path: &file.key.path,
            }),
        };
        let Some(text) = text else {
            continue;
        };
        let name = virtual_name(&file.key);
        files.keys.insert(name.clone(), file.key.clone());
        entries.push(InputFile {
            name,
            text,
            imports: file
                .resolved
                .iter()
                .filter(|(_, target)| is_code_file(&target.path))
                .map(|(specifier, target)| (specifier.clone(), virtual_name(target)))
                .collect(),
        });
    }
    let json = serde_json::to_vec(&Input {
        typescript: compiler,
        files: entries,
    })
    .map_err(|error| Error::new(format!("could not build the TypeScript input: {error}")))?;
    Ok((json, files))
}

fn virtual_name(key: &FileKey) -> String {
    match key.root {
        Root::Repository => format!("{REPOSITORY}{}", key.path),
        Root::Package(package) => format!("{PACKAGE}{package}/{}", key.path),
    }
}
