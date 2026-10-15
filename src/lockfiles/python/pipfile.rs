use serde_json::Value;

use super::{PythonLock, PythonPackage, PythonSource, normalized, sha256_of};
use crate::Error;

const PUBLIC_HOST: &str = "https://pypi.org/";
const SECTIONS: &[&str] = &["default", "develop"];

pub fn parse(text: &str) -> Result<PythonLock, Error> {
    let document: Value =
        serde_json::from_str(text).map_err(|error| Error::new(error.to_string()))?;
    let private_indexes: Vec<(&str, &str)> = document
        .pointer("/_meta/sources")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|source| Some((source.get("name")?.as_str()?, source.get("url")?.as_str()?)))
        .filter(|(_, url)| !url.starts_with(PUBLIC_HOST))
        .collect();
    let mut lock = PythonLock::default();
    for section in SECTIONS {
        for (name, entry) in document
            .get(*section)
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
        {
            let text = |field: &str| entry.get(field).and_then(Value::as_str);
            let version = text("version")
                .unwrap_or_default()
                .trim_start_matches("==")
                .to_string();
            let private = text("index")
                .and_then(|index| private_indexes.iter().find(|(name, _)| *name == index));
            let source = if let Some(url) = text("git") {
                PythonSource::Git(format!("{url}#{}", text("ref").unwrap_or_default()))
            } else if let Some(folder) = text("path") {
                PythonSource::Folder(folder.to_string())
            } else if let Some((_, url)) = private {
                PythonSource::Private(url.to_string())
            } else {
                PythonSource::Index {
                    hashes: entry
                        .get("hashes")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(|hash| hash.as_str().and_then(sha256_of))
                        .collect(),
                }
            };
            lock.packages.push(PythonPackage {
                name: normalized(name),
                version,
                source,
            });
        }
    }
    Ok(lock)
}
