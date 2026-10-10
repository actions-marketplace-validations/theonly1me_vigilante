use super::{PythonLock, PythonLockText, PythonPackage, normalized};

mod tables;
mod text;

impl PythonLock {
    fn package(&self, name: &str) -> Option<&PythonPackage> {
        let wanted = normalized(name);
        self.packages.iter().find(|package| package.name == wanted)
    }
}

struct Lockfile<'a> {
    path: &'a str,
    text: &'a str,
}

fn parse(lockfile: Lockfile) -> PythonLock {
    let Lockfile { path, text } = lockfile;
    PythonLock::parse(PythonLockText { path, text }).expect("lockfile")
}
