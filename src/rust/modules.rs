use super::environment::RustEnvironment;
use crate::deep::program::FileKey;
use crate::repository_path::{Folder, parent_folder};

const MODULE_FILE: &str = "mod.rs";

pub struct ModuleTree<'t, 'e> {
    pub environment: &'t RustEnvironment<'e>,
    pub root_file: &'t FileKey,
}

pub struct ChildModule<'a> {
    pub file: &'a FileKey,
    pub name: &'a str,
}

impl ModuleTree<'_, '_> {
    pub fn child(&self, wanted: ChildModule) -> Option<FileKey> {
        let ChildModule { file, name } = wanted;
        let folder = self.folder_of(file);
        [format!("{name}.rs"), format!("{name}/{MODULE_FILE}")]
            .iter()
            .filter_map(|relative| folder.joined(relative))
            .map(|path| FileKey {
                root: file.root.clone(),
                path,
            })
            .find(|candidate| self.environment.exists(candidate))
    }

    pub fn parent(&self, file: &FileKey) -> Option<FileKey> {
        if file == self.root_file {
            return None;
        }
        let container = if is_module_file(&file.path) {
            parent_folder(parent_folder(&file.path))
        } else {
            parent_folder(&file.path)
        };
        if container == parent_folder(&self.root_file.path) {
            return Some(self.root_file.clone());
        }
        [
            format!("{container}.rs"),
            format!("{container}/{MODULE_FILE}"),
        ]
        .into_iter()
        .map(|path| FileKey {
            root: file.root.clone(),
            path,
        })
        .find(|candidate| self.environment.exists(candidate))
    }

    fn folder_of(&self, file: &FileKey) -> String {
        let folder = parent_folder(&file.path);
        if file == self.root_file || is_module_file(&file.path) {
            return folder.to_string();
        }
        let stem = file
            .path
            .rsplit('/')
            .next()
            .unwrap_or_default()
            .trim_end_matches(".rs");
        folder.joined(stem).unwrap_or_default()
    }
}

fn is_module_file(path: &str) -> bool {
    path.rsplit('/').next() == Some(MODULE_FILE)
}
