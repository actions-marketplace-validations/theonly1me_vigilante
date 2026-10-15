use serde::Deserialize;

use crate::Error;

#[derive(Deserialize)]
struct LockFile {
    #[serde(default, rename = "package")]
    packages: Vec<RawPackage>,
}

#[derive(Deserialize)]
struct RawPackage {
    name: String,
    version: String,
    source: Option<String>,
    checksum: Option<String>,
    #[serde(default)]
    dependencies: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub source: Option<String>,
    pub checksum: Option<String>,
    pub dependencies: Vec<LockedDependency>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockedDependency {
    pub name: String,
    pub version: Option<String>,
}

pub fn read_lockfile(text: &str) -> Result<Vec<LockedPackage>, Error> {
    let file: LockFile =
        toml::from_str(text).map_err(|error| Error::new(format!("Cargo.lock: {error}")))?;
    Ok(file
        .packages
        .into_iter()
        .map(|package| LockedPackage {
            name: package.name,
            version: package.version,
            source: package.source,
            checksum: package.checksum,
            dependencies: package
                .dependencies
                .iter()
                .filter_map(|entry| {
                    let mut parts = entry.split_whitespace();
                    Some(LockedDependency {
                        name: parts.next()?.to_string(),
                        version: parts.next().map(str::to_string),
                    })
                })
                .collect(),
        })
        .collect())
}
