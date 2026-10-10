use toml::Value;

use super::{PythonLock, PythonPackage, PythonSource, normalized, sha256_of};
use crate::Error;

pub fn parse(text: &str) -> Result<PythonLock, Error> {
    let document: Value = toml::from_str(text).map_err(|error| Error::new(error.to_string()))?;
    let mut lock = PythonLock::default();
    for entry in document
        .get("package")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let text = |field: &str| entry.get(field).and_then(Value::as_str);
        let (Some(name), Some(version)) = (text("name"), text("version")) else {
            continue;
        };
        lock.packages.push(PythonPackage {
            name: normalized(name),
            version: version.to_string(),
            source: source_of(entry),
        });
    }
    Ok(lock)
}

fn source_of(entry: &Value) -> PythonSource {
    let source = entry.get("source");
    let field = |name: &str| {
        source
            .and_then(|source| source.get(name))
            .and_then(Value::as_str)
    };
    match field("type") {
        Some("git") => {
            let url = field("url").unwrap_or_default();
            let commit = field("resolved_reference")
                .or_else(|| field("reference"))
                .unwrap_or_default();
            PythonSource::Git(format!("{url}#{commit}"))
        }
        Some("directory") => PythonSource::Folder(field("url").unwrap_or_default().to_string()),
        Some("legacy") => PythonSource::Private(field("url").unwrap_or_default().to_string()),
        Some(other) => {
            PythonSource::Unsupported(format!("{other} {}", field("url").unwrap_or_default()))
        }
        None => PythonSource::Index {
            hashes: entry
                .get("files")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|file| file.get("hash")?.as_str().and_then(sha256_of))
                .collect(),
        },
    }
}
