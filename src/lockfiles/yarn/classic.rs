use std::collections::HashMap;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use super::{YarnLockfile, YarnPackage, split_descriptor, unquoted};
use crate::lockfiles::{LockedPackage, PackageSource};

const DEPENDENCY_SECTIONS: &[&str] = &["dependencies", "optionalDependencies"];

#[derive(Default)]
struct Entry {
    descriptors: Vec<String>,
    fields: HashMap<String, String>,
    dependencies: HashMap<String, String>,
}

pub fn parse(text: &str) -> YarnLockfile {
    let mut lockfile = YarnLockfile::default();
    let mut entry: Option<Entry> = None;
    let mut section: Option<String> = None;
    for line in text.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let content = line.trim();
        match indent {
            0 => {
                if let Some(done) = entry.take() {
                    lockfile.add_classic(done);
                }
                let descriptors = content
                    .trim_end_matches(':')
                    .split(", ")
                    .map(|descriptor| unquoted(descriptor).to_string())
                    .collect();
                entry = Some(Entry {
                    descriptors,
                    ..Entry::default()
                });
                section = None;
            }
            2 if content.ends_with(':') => {
                section = Some(content.trim_end_matches(':').to_string());
            }
            2 => {
                section = None;
                if let (Some(entry), Some((field, value))) =
                    (entry.as_mut(), content.split_once(char::is_whitespace))
                {
                    entry
                        .fields
                        .insert(field.to_string(), unquoted(value).to_string());
                }
            }
            _ => {
                let is_dependency = section
                    .as_deref()
                    .is_some_and(|name| DEPENDENCY_SECTIONS.contains(&name));
                if let (true, Some(entry), Some((name, range))) = (
                    is_dependency,
                    entry.as_mut(),
                    content.split_once(char::is_whitespace),
                ) {
                    let name = unquoted(name);
                    entry
                        .dependencies
                        .insert(name.to_string(), format!("{name}@{}", unquoted(range)));
                }
            }
        }
    }
    if let Some(done) = entry.take() {
        lockfile.add_classic(done);
    }
    lockfile
}

impl YarnLockfile {
    fn add_classic(&mut self, entry: Entry) {
        add(Classic {
            lockfile: self,
            entry,
        });
    }
}

struct Classic<'a> {
    lockfile: &'a mut YarnLockfile,
    entry: Entry,
}

fn add(classic: Classic) {
    let Classic { lockfile, entry } = classic;
    let Some((written_name, range)) = entry
        .descriptors
        .first()
        .and_then(|first| split_descriptor(first))
    else {
        return;
    };
    let name = range
        .strip_prefix("npm:")
        .and_then(split_descriptor)
        .map_or(written_name, |(real, _)| real)
        .to_string();
    let version = entry.fields.get("version").cloned().unwrap_or_default();
    let source = source_of(&entry.fields);
    let key = entry.descriptors.first().cloned().unwrap_or_default();
    lockfile.add(YarnPackage {
        descriptors: entry.descriptors,
        key,
        package: LockedPackage {
            name,
            version,
            source,
        },
        dependencies: entry.dependencies,
    });
}

fn source_of(fields: &HashMap<String, String>) -> PackageSource {
    let Some(resolved) = fields.get("resolved") else {
        return PackageSource::Unsupported("a lockfile entry with no resolved address".to_string());
    };
    if resolved.starts_with("git+")
        || resolved.starts_with("git:")
        || resolved.starts_with("github:")
    {
        return PackageSource::Git(resolved.clone());
    }
    if let Some(folder) = resolved.strip_prefix("file:") {
        return PackageSource::Folder(folder.to_string());
    }
    if !resolved.starts_with("https://") && !resolved.starts_with("http://") {
        return PackageSource::Unsupported(resolved.clone());
    }
    let (url, fragment) = resolved.split_once('#').unwrap_or((resolved, ""));
    let integrity = fields
        .get("integrity")
        .cloned()
        .or_else(|| sha1_integrity(fragment));
    PackageSource::Registry {
        url: url.to_string(),
        integrity,
    }
}

fn sha1_integrity(hex: &str) -> Option<String> {
    if hex.len() != 40 {
        return None;
    }
    let bytes: Option<Vec<u8>> = (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(hex.get(index..index + 2)?, 16).ok())
        .collect();
    Some(format!("sha1-{}", STANDARD.encode(bytes?)))
}
