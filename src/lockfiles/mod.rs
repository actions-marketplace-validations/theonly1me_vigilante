mod npm;

pub use npm::{NpmLockfile, PackageSource};

pub const NPM_LOCKFILES: &[&str] = &["npm-shrinkwrap.json", "package-lock.json"];
