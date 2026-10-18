use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::config::Config;

const FILES: &[&str] = &["vigilante.lock", Config::FILE];
const FOLDERS: &[(&str, &str)] = &[
    (".vigilante", "rules/rule.yml"),
    (".github/workflows", "vigilante.yml"),
];
const RULE_SAMPLE: &str = "rule.yml";

pub struct Protected {
    pub label: String,
    pub sample: String,
    pub files: Vec<String>,
}

pub struct ProtectedPaths {
    pub entries: Vec<Protected>,
    pub problem: Option<String>,
}

pub fn protected_paths(repository: &Path) -> ProtectedPaths {
    let mut entries: Vec<Protected> = FILES
        .iter()
        .map(|file| Protected {
            label: file.to_string(),
            sample: file.to_string(),
            files: Vec::new(),
        })
        .collect();
    let (rule_folders, problem) = match rule_folders(repository) {
        Ok(folders) => (folders, None),
        Err(problem) => (Vec::new(), Some(problem)),
    };
    let folders = FOLDERS
        .iter()
        .map(|(folder, sample)| (folder.to_string(), sample.to_string()))
        .chain(
            rule_folders
                .into_iter()
                .map(|folder| (folder, RULE_SAMPLE.to_string())),
        );
    for (folder, sample) in folders {
        let label = format!("{folder}/");
        if entries.iter().any(|entry| entry.label == label) {
            continue;
        }
        entries.push(Protected {
            sample: format!("{label}{sample}"),
            files: files_under(&FolderQuery {
                repository,
                folder: &folder,
            }),
            label,
        });
    }
    ProtectedPaths { entries, problem }
}

fn rule_folders(repository: &Path) -> Result<Vec<String>, String> {
    let text = match fs::read_to_string(repository.join(Config::FILE)) {
        Ok(text) => text,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("cannot read {}: {error}", Config::FILE)),
    };
    let config = Config::parse(&text).map_err(|error| error.to_string())?;
    Ok(config
        .rule_paths
        .iter()
        .map(|folder| folder.trim_end_matches('/').to_string())
        .collect())
}

struct FolderQuery<'a> {
    repository: &'a Path,
    folder: &'a str,
}

fn files_under(query: &FolderQuery) -> Vec<String> {
    let mut files = Vec::new();
    let mut pending = vec![query.folder.to_string()];
    while let Some(folder) = pending.pop() {
        let Ok(entries) = fs::read_dir(query.repository.join(&folder)) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = format!("{folder}/{}", entry.file_name().to_string_lossy());
            match entry.file_type() {
                Ok(kind) if kind.is_dir() => pending.push(path),
                Ok(kind) if kind.is_file() => files.push(path),
                _ => {}
            }
        }
    }
    files.sort();
    files
}
