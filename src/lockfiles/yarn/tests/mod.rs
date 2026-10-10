use crate::lockfiles::{Found, JavaScriptLockfile, LockfileText, Lookup, PackageSource, Place};

mod berry;
mod classic;

fn yarn(text: &str) -> JavaScriptLockfile {
    JavaScriptLockfile::parse(LockfileText {
        path: "yarn.lock",
        text,
    })
    .expect("lockfile")
}

struct Query<'a> {
    lockfile: &'a JavaScriptLockfile,
    lookup: Lookup<'a>,
}

struct Declared<'a> {
    name: &'a str,
    declared: Option<&'a str>,
}

struct Tarball<'a> {
    url: &'a str,
    integrity: Option<&'a str>,
}

fn found(query: Query) -> (String, String, PackageSource) {
    let Query { lockfile, lookup } = query;
    let Found { key, package } = lockfile.find(lookup).expect("found");
    (
        key,
        format!("{}@{}", package.name, package.version),
        package.source,
    )
}

fn from_repository(wanted: Declared) -> Lookup {
    let Declared { name, declared } = wanted;
    Lookup {
        place: Place::Repository { folder: "src" },
        name,
        declared,
    }
}

fn registry(tarball: Tarball) -> PackageSource {
    let Tarball { url, integrity } = tarball;
    PackageSource::Registry {
        url: url.to_string(),
        integrity: integrity.map(str::to_string),
    }
}
