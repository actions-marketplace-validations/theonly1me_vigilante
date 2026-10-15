use crate::lockfiles::{Found, JavaScriptLockfile, LockfileText, Lookup, PackageSource, Place};

mod older_versions;
mod version_9;

fn pnpm(text: &str) -> JavaScriptLockfile {
    JavaScriptLockfile::parse(LockfileText {
        path: "pnpm-lock.yaml",
        text,
    })
    .expect("lockfile")
}

struct Published<'a> {
    name: &'a str,
    version: &'a str,
    integrity: &'a str,
}

struct Query<'a> {
    lockfile: &'a JavaScriptLockfile,
    place: Place<'a>,
    name: &'a str,
}

fn registry(published: Published) -> PackageSource {
    let Published {
        name,
        version,
        integrity,
    } = published;
    let base_name = name.rsplit('/').next().unwrap_or(name);
    PackageSource::Registry {
        url: format!("https://registry.npmjs.org/{name}/-/{base_name}-{version}.tgz"),
        integrity: Some(integrity.to_string()),
    }
}

fn found(query: Query) -> (String, String, PackageSource) {
    let Query {
        lockfile,
        place,
        name,
    } = query;
    let Found { key, package } = lockfile
        .find(Lookup {
            place,
            name,
            declared: None,
        })
        .expect("found");
    (
        key,
        format!("{}@{}", package.name, package.version),
        package.source,
    )
}
