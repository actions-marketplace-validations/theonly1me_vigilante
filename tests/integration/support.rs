use std::collections::BTreeSet;
use std::fs;

use tempfile::TempDir;
use vigilante::{PowerMap, ScanRequest, Source};

pub fn write_files(files: &[(&str, &str)]) -> TempDir {
    let directory = tempfile::tempdir().expect("temporary directory");
    for (path, text) in files {
        let target = directory.path().join(path);
        fs::create_dir_all(target.parent().expect("parent folder")).expect("folders");
        fs::write(target, text).expect("file");
    }
    directory
}

pub fn scan_files(files: &[(&str, &str)]) -> PowerMap {
    let directory = write_files(files);
    vigilante::scan(ScanRequest {
        repository: directory.path().to_path_buf(),
        source: Source::WorkingDirectory,
    })
    .expect("scan")
}

pub fn sightings(map: &PowerMap) -> BTreeSet<String> {
    map.scopes
        .iter()
        .flat_map(|(scope, powers)| {
            powers.iter().map(move |(power, evidence)| {
                let first = evidence.first().expect("evidence");
                format!(
                    "{} {} {power} {}:{}",
                    scope.surface.name(),
                    scope.path,
                    first.path,
                    first.line
                )
            })
        })
        .collect()
}

pub fn expected(lines: &[&str]) -> BTreeSet<String> {
    lines.iter().map(|line| line.to_string()).collect()
}
