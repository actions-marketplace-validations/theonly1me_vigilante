mod npm;
mod pnpm;
#[cfg(test)]
mod tests;

use npm::NpmLockfile;
use pnpm::PnpmLockfile;

use crate::Error;

pub const JAVASCRIPT_LOCKFILES: &[&str] =
    &["npm-shrinkwrap.json", "package-lock.json", "pnpm-lock.yaml"];
const DEFAULT_NPM_REGISTRY: &str = "https://registry.npmjs.org";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageSource {
    Registry {
        url: String,
        integrity: Option<String>,
    },
    Folder(String),
    Git(String),
    Bundled,
    Unsupported(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub source: PackageSource,
}

#[derive(Clone, Copy, Debug)]
pub enum Place<'a> {
    Repository { folder: &'a str },
    Package { key: &'a str },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub key: String,
    pub package: LockedPackage,
}

pub enum JavaScriptLockfile {
    Npm(NpmLockfile),
    Pnpm(PnpmLockfile),
}

impl JavaScriptLockfile {
    pub fn parse(path: &str, text: &str) -> Result<Self, Error> {
        if path.ends_with("pnpm-lock.yaml") {
            PnpmLockfile::parse(text).map(Self::Pnpm)
        } else {
            NpmLockfile::parse(text).map(Self::Npm)
        }
    }

    pub fn find(&self, place: Place, name: &str) -> Option<Found> {
        match self {
            Self::Npm(lockfile) => lockfile.find(place, name),
            Self::Pnpm(lockfile) => lockfile.find(place, name),
        }
    }
}

fn default_url(name: &str, version: &str) -> String {
    let base_name = name.rsplit('/').next().unwrap_or(name);
    format!("{DEFAULT_NPM_REGISTRY}/{name}/-/{base_name}-{version}.tgz")
}
