use super::{Lockfile, parse};
use crate::lockfiles::python::PythonSource;

const PIPFILE: &str = r#"{
  "_meta": {"sources": [{"name": "pypi", "url": "https://pypi.org/simple"}, {"name": "acme", "url": "https://pypi.acme.dev/simple"}]},
  "default": {"requests": {"hashes": ["sha256:dddd"], "index": "pypi", "version": "==2.31.0"}, "inside": {"index": "acme", "version": "==2.0.0"}},
  "develop": {"forked": {"git": "https://github.com/acme/forked.git", "ref": "0123456789abcdef"}}
}"#;

const REQUIREMENTS: &str = "# pinned\nrequests[socks]==2.31.0 ; python_version >= '3.8' \\\n    --hash=sha256:eeee \\\n    --hash=sha256:ffff\nflask>=2.0\n-r other.txt\ngit+https://github.com/acme/forked.git@0123456789abcdef#egg=forked\n";

#[test]
fn pipfile_reads_both_sections_and_private_indexes() {
    let lock = parse(Lockfile {
        path: "Pipfile.lock",
        text: PIPFILE,
    });

    let requests = lock.package("requests").expect("requests");
    assert_eq!(requests.version, "2.31.0");
    assert_eq!(
        requests.source,
        PythonSource::Index {
            hashes: vec!["dddd".to_string()]
        }
    );
    assert_eq!(
        lock.package("inside").map(|package| &package.source),
        Some(&PythonSource::Private(
            "https://pypi.acme.dev/simple".to_string()
        ))
    );
    assert_eq!(
        lock.package("forked").map(|package| &package.source),
        Some(&PythonSource::Git(
            "https://github.com/acme/forked.git#0123456789abcdef".to_string()
        ))
    );
}

#[test]
fn requirements_reads_pins_hashes_git_and_lists_unpinned_lines() {
    let lock = parse(Lockfile {
        path: "requirements.txt",
        text: REQUIREMENTS,
    });

    let requests = lock.package("requests").expect("requests");
    assert_eq!(requests.version, "2.31.0");
    assert_eq!(
        requests.source,
        PythonSource::Index {
            hashes: vec!["eeee".to_string(), "ffff".to_string()]
        }
    );
    assert_eq!(
        lock.package("forked").map(|package| &package.source),
        Some(&PythonSource::Git(
            "https://github.com/acme/forked.git#0123456789abcdef".to_string()
        ))
    );
    assert_eq!(lock.unpinned, vec!["flask".to_string()]);
}
