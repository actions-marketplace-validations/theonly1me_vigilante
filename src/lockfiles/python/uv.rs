use toml::Value;

use super::{
    PUBLIC_INDEX, PythonFile, PythonLock, PythonPackage, PythonSource, normalized, sha256_of,
};
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
    for local in ["editable", "virtual", "directory", "path"] {
        if let Some(folder) = field(local) {
            return PythonSource::Folder(folder.to_string());
        }
    }
    if let Some(git) = field("git") {
        return PythonSource::Git(git.to_string());
    }
    if let Some(registry) = field("registry")
        && registry.trim_end_matches('/') != PUBLIC_INDEX
    {
        return PythonSource::Private(registry.to_string());
    }
    let file = |item: &Value| {
        Some(PythonFile {
            url: item.get("url")?.as_str()?.to_string(),
            sha256: item.get("hash").and_then(Value::as_str).and_then(sha256_of),
            size: item
                .get("size")
                .and_then(Value::as_integer)
                .and_then(|size| u64::try_from(size).ok()),
        })
    };
    let mut files: Vec<PythonFile> = entry
        .get("wheels")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(file)
        .collect();
    files.extend(entry.get("sdist").and_then(file));
    if files.is_empty() {
        return PythonSource::Unsupported(
            field("url")
                .unwrap_or("a package with no files")
                .to_string(),
        );
    }
    PythonSource::Files(files)
}
