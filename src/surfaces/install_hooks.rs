use std::collections::BTreeSet;

use crate::document::Node;
use crate::{Capability, Detection, Power};

const INSTALL_SCRIPTS: &[&str] = &[
    "preinstall",
    "install",
    "postinstall",
    "prepublish",
    "preprepare",
    "prepare",
    "postprepare",
];
const PRE_COMMIT_CONFIG: &str = ".pre-commit-config.yaml";
const PRE_COMMIT_BUILT_IN_REPOSITORIES: &[&str] = &["meta"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HookFile<'a> {
    PackageManifest,
    BuildScript,
    SetupScript,
    HuskyHook { name: &'a str },
    PreCommitConfig,
}

pub struct HookPath<'a> {
    pub path: &'a str,
    pub paths: &'a BTreeSet<String>,
}

pub fn kind_of(hook_path: HookPath<'_>) -> Option<HookFile<'_>> {
    let path = hook_path.path;
    let (folder, name) = path.rsplit_once('/').unwrap_or(("", path));
    let folder_name = folder.rsplit('/').next().unwrap_or(folder);
    match name {
        "package.json" => Some(HookFile::PackageManifest),
        "setup.py" => Some(HookFile::SetupScript),
        PRE_COMMIT_CONFIG => Some(HookFile::PreCommitConfig),
        "build.rs"
            if hook_path.paths.contains(&sibling(Sibling {
                folder,
                name: "Cargo.toml",
            })) =>
        {
            Some(HookFile::BuildScript)
        }
        _ if folder_name == ".husky" && !name.starts_with('.') => {
            Some(HookFile::HuskyHook { name })
        }
        _ => None,
    }
}

pub fn detect_static(kind: HookFile) -> Vec<Detection> {
    let power = match kind {
        HookFile::BuildScript => Power::new(Capability::InstallHook).with_detail("build.rs"),
        HookFile::SetupScript => Power::new(Capability::InstallHook).with_detail("setup.py"),
        HookFile::HuskyHook { name } => Power::new(Capability::GitHook).with_detail(name),
        HookFile::PackageManifest | HookFile::PreCommitConfig => return Vec::new(),
    };
    vec![detection(Found { power, line: 1 })]
}

pub fn detect_package_manifest(manifest: &Node) -> Vec<Detection> {
    manifest
        .get("scripts")
        .map(Node::entries)
        .unwrap_or_default()
        .iter()
        .filter(|entry| INSTALL_SCRIPTS.contains(&entry.key.as_str()))
        .map(|entry| {
            let command = entry.value.text().unwrap_or_default();
            detection(Found {
                power: Power::new(Capability::InstallHook)
                    .with_detail(format!("{}: {command}", entry.key)),
                line: entry.line,
            })
        })
        .collect()
}

pub fn detect_pre_commit(config: &Node) -> Vec<Detection> {
    let mut found = Vec::new();
    for repository in config.get("repos").map(Node::items).unwrap_or_default() {
        let Some(url) = repository.get("repo").and_then(Node::text) else {
            continue;
        };
        if PRE_COMMIT_BUILT_IN_REPOSITORIES.contains(&url) {
            continue;
        }
        if url != "local" {
            found.push(detection(Found {
                power: Power::new(Capability::GitHook).with_detail(url),
                line: repository.line,
            }));
            continue;
        }
        for hook in repository.get("hooks").map(Node::items).unwrap_or_default() {
            let entry = hook.get("entry").and_then(Node::text).unwrap_or_default();
            found.push(detection(Found {
                power: Power::new(Capability::GitHook).with_detail(format!("local: {entry}")),
                line: hook.line,
            }));
        }
    }
    found
}

struct Found {
    power: Power,
    line: usize,
}

fn detection(found: Found) -> Detection {
    Detection {
        power: found.power,
        line: found.line,
        rule: "install-hook".to_string(),
    }
}

struct Sibling<'a> {
    folder: &'a str,
    name: &'a str,
}

fn sibling(target: Sibling) -> String {
    if target.folder.is_empty() {
        target.name.to_string()
    } else {
        format!("{}/{}", target.folder, target.name)
    }
}
