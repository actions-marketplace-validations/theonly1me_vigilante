use super::RustEnvironment;
use super::lookup::CrateId;
use crate::deep::program::{FileKey, Root};
use crate::repository_path::Folder;

const LIBRARY: &str = "src/lib.rs";
const MAIN: &str = "src/main.rs";
const BUILD_SCRIPT: &str = "build.rs";
const TARGET_FOLDERS: &[&str] = &["src/bin/", "examples/", "tests/", "benches/"];

pub struct CrateRoot {
    pub id: CrateId,
    pub file: FileKey,
}

impl RustEnvironment<'_> {
    pub fn crate_root(&self, file: &FileKey) -> Option<CrateRoot> {
        if let Root::Package(package) = file.root {
            let id = CrateId::Package(package);
            return Some(CrateRoot {
                file: self.library_root(id)?,
                id,
            });
        }
        let (position, own) = self
            .own
            .iter()
            .enumerate()
            .filter(|(_, own)| {
                own.folder.is_empty() || file.path.starts_with(&format!("{}/", own.folder))
            })
            .max_by_key(|(_, own)| own.folder.len())?;
        let relative = file
            .path
            .strip_prefix(&own.folder)
            .unwrap_or(&file.path)
            .trim_start_matches('/');
        let id = CrateId::Own(position);
        let root = match target_root(relative) {
            Some(target) => FileKey {
                root: Root::Repository,
                path: own.folder.joined(&target)?,
            },
            None => self.library_root(id)?,
        };
        Some(CrateRoot { id, file: root })
    }

    pub fn library_root(&self, id: CrateId) -> Option<FileKey> {
        match id {
            CrateId::Package(package) => Some(FileKey {
                root: Root::Package(package),
                path: library_path(self.installed[package].manifest.library.as_deref()),
            }),
            CrateId::Own(position) => {
                let own = &self.own[position];
                [
                    library_path(own.manifest.library.as_deref()),
                    MAIN.to_string(),
                ]
                .iter()
                .filter_map(|relative| own.folder.joined(relative))
                .find(|path| self.tree.paths().contains(path))
                .map(|path| FileKey {
                    root: Root::Repository,
                    path,
                })
            }
        }
    }
}

fn library_path(declared: Option<&str>) -> String {
    declared
        .map(|path| path.trim_start_matches("./").to_string())
        .unwrap_or_else(|| LIBRARY.to_string())
}

fn target_root(relative: &str) -> Option<String> {
    if relative == BUILD_SCRIPT || relative == MAIN {
        return Some(relative.to_string());
    }
    TARGET_FOLDERS.iter().find_map(|folder| {
        let rest = relative.strip_prefix(folder)?;
        Some(match rest.split_once('/') {
            Some((first, _)) => format!("{folder}{first}/main.rs"),
            None => relative.to_string(),
        })
    })
}
