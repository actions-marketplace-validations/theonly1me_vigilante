use std::collections::HashMap;

const SPEC_INDENT: &str = "    ";
const DEPENDENCY_INDENT: &str = "      ";
const CHECKSUM_PREFIX: &str = "sha256=";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GemSource {
    Registry { remote: String },
    Git { remote: String, revision: String },
    Path { folder: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockedGem {
    pub name: String,
    pub version: String,
    pub platform: Option<String>,
    pub source: GemSource,
    pub checksum: Option<String>,
    pub dependencies: Vec<String>,
}

#[derive(Default)]
struct Section {
    kind: String,
    remote: String,
    revision: String,
}

pub fn read_lockfile(text: &str) -> Vec<LockedGem> {
    let mut section = Section::default();
    let mut gems: Vec<LockedGem> = Vec::new();
    let mut checksums: HashMap<String, String> = HashMap::new();
    for line in text.lines() {
        if !line.starts_with(' ') && !line.trim().is_empty() {
            section = Section {
                kind: line.trim().to_string(),
                ..Section::default()
            };
            continue;
        }
        let trimmed = line.trim();
        if section.kind == "CHECKSUMS" {
            if let Some((spec, checksum)) = trimmed.split_once(' ').and_then(|(name, rest)| {
                let (version, checksum) = rest.split_once(") ")?;
                Some((format!("{name} {})", version), checksum))
            }) && let Some(hex) = checksum.strip_prefix(CHECKSUM_PREFIX)
            {
                checksums.insert(spec, hex.to_string());
            }
            continue;
        }
        if !matches!(section.kind.as_str(), "GEM" | "GIT" | "PATH") {
            continue;
        }
        if let Some(remote) = trimmed.strip_prefix("remote: ") {
            section.remote = remote.to_string();
        } else if let Some(revision) = trimmed.strip_prefix("revision: ") {
            section.revision = revision.to_string();
        } else if line.starts_with(DEPENDENCY_INDENT) {
            if let (Some(gem), Some(name)) = (gems.last_mut(), trimmed.split_whitespace().next()) {
                gem.dependencies.push(name.to_string());
            }
        } else if line.starts_with(SPEC_INDENT)
            && let Some((name, rest)) = trimmed.split_once(" (")
        {
            let full = rest.trim_end_matches(')');
            let (version, platform) = match full.split_once('-') {
                Some((version, platform)) => (version.to_string(), Some(platform.to_string())),
                None => (full.to_string(), None),
            };
            gems.push(LockedGem {
                name: name.to_string(),
                version,
                platform,
                source: match section.kind.as_str() {
                    "GIT" => GemSource::Git {
                        remote: section.remote.clone(),
                        revision: section.revision.clone(),
                    },
                    "PATH" => GemSource::Path {
                        folder: section.remote.clone(),
                    },
                    _ => GemSource::Registry {
                        remote: section.remote.clone(),
                    },
                },
                checksum: None,
                dependencies: Vec::new(),
            });
        }
    }
    let mut chosen: Vec<LockedGem> = Vec::new();
    for mut gem in gems {
        let spec = format!(
            "{} ({}{})",
            gem.name,
            gem.version,
            gem.platform
                .as_ref()
                .map(|platform| format!("-{platform}"))
                .unwrap_or_default()
        );
        gem.checksum = checksums.get(&spec).cloned();
        match chosen
            .iter_mut()
            .find(|known| known.name == gem.name && known.source == gem.source)
        {
            Some(known) if known.platform.is_some() && gem.platform.is_none() => *known = gem,
            Some(_) => {}
            None => chosen.push(gem),
        }
    }
    chosen
}
