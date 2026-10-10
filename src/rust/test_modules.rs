use std::collections::BTreeSet;

use ast_grep_core::tree_sitter::LanguageExt;
use ast_grep_language::SupportLang;

use super::environment::RustEnvironment;
use super::modules::{ChildModule, ModuleTree};
use super::syntax::test_module_names;
use crate::Error;
use crate::deep::program::{FileKey, Root};
use crate::tree::Tree;

const MODULE_FILE_SUFFIX: &str = "/mod.rs";
const SOURCE_SUFFIX: &str = ".rs";

pub struct TestModulesInput<'a, 'e> {
    pub tree: &'a dyn Tree,
    pub environment: &'a RustEnvironment<'e>,
    pub roots: BTreeSet<String>,
}

pub fn without_test_modules(input: TestModulesInput) -> Result<BTreeSet<String>, Error> {
    let TestModulesInput {
        tree,
        environment,
        roots,
    } = input;
    let paths: Vec<&str> = roots.iter().map(String::as_str).collect();
    let mut test_files = BTreeSet::new();
    let mut test_folders = Vec::new();
    for file in tree.read(&paths)? {
        let key = FileKey {
            root: Root::Repository,
            path: file.path,
        };
        let Some(crate_root) = environment.crate_root(&key) else {
            continue;
        };
        let modules = ModuleTree {
            environment,
            root_file: &crate_root.file,
        };
        let document = SupportLang::Rust.ast_grep(&file.text);
        for name in test_module_names(&document.root()) {
            let Some(child) = modules.child(ChildModule {
                file: &key,
                name: &name,
            }) else {
                continue;
            };
            let folder = child
                .path
                .strip_suffix(MODULE_FILE_SUFFIX)
                .or_else(|| child.path.strip_suffix(SOURCE_SUFFIX))
                .unwrap_or(&child.path);
            test_folders.push(format!("{folder}/"));
            test_files.insert(child.path);
        }
    }
    Ok(roots
        .into_iter()
        .filter(|path| {
            !test_files.contains(path)
                && !test_folders.iter().any(|folder| path.starts_with(folder))
        })
        .collect())
}
