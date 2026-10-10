use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::Error;

pub fn package_root(folder: &Path, manifest: &str) -> PathBuf {
    if folder.join(manifest).is_file() {
        return folder.to_path_buf();
    }
    let Ok(entries) = fs::read_dir(folder) else {
        return folder.to_path_buf();
    };
    let mut children: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.join(manifest).is_file())
        .collect();
    children.sort();
    children
        .into_iter()
        .next()
        .unwrap_or_else(|| folder.to_path_buf())
}

pub fn folder_files(root: &Path) -> Result<BTreeSet<String>, Error> {
    let mut files = BTreeSet::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(folder) = pending.pop() {
        let entries = fs::read_dir(&folder)
            .map_err(|error| Error::new(format!("cannot read {}: {error}", folder.display())))?;
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() && entry.file_name() != "node_modules" {
                pending.push(path);
            } else if kind.is_file()
                && let Ok(relative) = path.strip_prefix(root)
            {
                files.insert(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    Ok(files)
}
