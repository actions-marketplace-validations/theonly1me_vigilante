mod pipfile;
mod poetry;
mod requirements;
#[cfg(test)]
mod tests;
mod uv;

use crate::Error;

pub const PYTHON_LOCKFILES: &[&str] =
    &["uv.lock", "poetry.lock", "Pipfile.lock", "requirements.txt"];
const PUBLIC_INDEX: &str = "https://pypi.org/simple";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PythonFile {
    pub url: String,
    pub sha256: Option<String>,
    pub size: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PythonSource {
    Files(Vec<PythonFile>),
    Index { hashes: Vec<String> },
    Git(String),
    Folder(String),
    Private(String),
    Unsupported(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PythonPackage {
    pub name: String,
    pub version: String,
    pub source: PythonSource,
}

#[derive(Debug, Default)]
pub struct PythonLock {
    pub packages: Vec<PythonPackage>,
    pub unpinned: Vec<String>,
}

pub struct PythonLockText<'a> {
    pub path: &'a str,
    pub text: &'a str,
}

impl PythonLock {
    pub fn parse(lockfile: PythonLockText) -> Result<Self, Error> {
        let name = lockfile.path.rsplit('/').next().unwrap_or(lockfile.path);
        let parsed = match name {
            "uv.lock" => uv::parse(lockfile.text).or_else(|error| {
                let compiled = requirements::parse(lockfile.text);
                if compiled.packages.is_empty() {
                    Err(error)
                } else {
                    Ok(compiled)
                }
            }),
            "poetry.lock" => poetry::parse(lockfile.text),
            "Pipfile.lock" => pipfile::parse(lockfile.text),
            _ => Ok(requirements::parse(lockfile.text)),
        };
        parsed.map_err(|error| Error::new(format!("{}: {error}", lockfile.path)))
    }
}

pub fn normalized(name: &str) -> String {
    let mut result = String::new();
    let mut previous_separator = false;
    for character in name.trim().chars() {
        if matches!(character, '-' | '_' | '.') {
            if !previous_separator {
                result.push('-');
            }
            previous_separator = true;
        } else {
            result.push(character.to_ascii_lowercase());
            previous_separator = false;
        }
    }
    result
}

fn sha256_of(hash: &str) -> Option<String> {
    hash.strip_prefix("sha256:")
        .or_else(|| hash.strip_prefix("sha256="))
        .map(str::to_string)
}
