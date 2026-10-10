mod npm;
mod pnpm;
#[cfg(test)]
mod tests;
mod yarn;

use std::collections::HashMap;

use npm::NpmLockfile;
use pnpm::PnpmLockfile;
use yarn::YarnLockfile;

use crate::Error;

pub const JAVASCRIPT_LOCKFILES: &[&str] = &[
    "npm-shrinkwrap.json",
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
];
pub const YARN_LOCKFILE: &str = "yarn.lock";
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

#[derive(Clone, Copy, Debug)]
pub struct Lookup<'a> {
    pub place: Place<'a>,
    pub name: &'a str,
    pub declared: Option<&'a str>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub key: String,
    pub package: LockedPackage,
}

pub enum JavaScriptLockfile {
    Npm(NpmLockfile),
    Pnpm(PnpmLockfile),
    Yarn(YarnLockfile),
}

impl JavaScriptLockfile {
    pub fn parse(path: &str, text: &str) -> Result<Self, Error> {
        if path.ends_with("pnpm-lock.yaml") {
            PnpmLockfile::parse(text).map(Self::Pnpm)
        } else if path.ends_with(YARN_LOCKFILE) {
            YarnLockfile::parse(text).map(Self::Yarn)
        } else {
            NpmLockfile::parse(text).map(Self::Npm)
        }
    }

    pub fn find(&self, lookup: Lookup) -> Option<Found> {
        match self {
            Self::Npm(lockfile) => lockfile.find(lookup),
            Self::Pnpm(lockfile) => lockfile.find(lookup),
            Self::Yarn(lockfile) => lockfile.find(lookup),
        }
    }

    pub fn add_workspaces(&mut self, workspaces: HashMap<String, String>) {
        if let Self::Yarn(lockfile) = self {
            lockfile.add_workspaces(workspaces);
        }
    }
}

fn default_url(name: &str, version: &str) -> String {
    let base_name = name.rsplit('/').next().unwrap_or(name);
    format!("{DEFAULT_NPM_REGISTRY}/{name}/-/{base_name}-{version}.tgz")
}
