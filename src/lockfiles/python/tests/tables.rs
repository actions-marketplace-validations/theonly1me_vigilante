use super::{Lockfile, parse};
use crate::lockfiles::python::{PythonFile, PythonSource};

const UV: &str = r#"version = 1

[[package]]
name = "app"
version = "0.1.0"
source = { editable = "." }

[[package]]
name = "Requests_Toolbelt"
version = "1.0.0"
source = { registry = "https://pypi.org/simple" }
sdist = { url = "https://files.pythonhosted.org/r/requests-toolbelt-1.0.0.tar.gz", hash = "sha256:aaaa" }
wheels = [{ url = "https://files.pythonhosted.org/r/requests_toolbelt-1.0.0-py3-none-any.whl", hash = "sha256:bbbb" }]

[[package]]
name = "inside"
version = "2.0.0"
source = { registry = "https://pypi.acme.dev/simple" }

[[package]]
name = "forked"
version = "0.1.0"
source = { git = "https://github.com/acme/forked?rev=main#0123456789abcdef" }
"#;

const POETRY: &str = r#"[[package]]
name = "pyyaml"
version = "6.0.1"
files = [
    {file = "PyYAML-6.0.1.tar.gz", hash = "sha256:cccc"},
]

[[package]]
name = "forked"
version = "0.1.0"

[package.source]
type = "git"
url = "https://github.com/acme/forked.git"
reference = "main"
resolved_reference = "0123456789abcdef"

[[package]]
name = "inside"
version = "2.0.0"

[package.source]
type = "legacy"
url = "https://pypi.acme.dev/simple"
reference = "acme"
"#;

#[test]
fn uv_reads_files_folders_private_indexes_and_git() {
    let lock = parse(Lockfile {
        path: "uv.lock",
        text: UV,
    });

    assert_eq!(
        lock.package("requests-toolbelt")
            .map(|package| &package.source),
        Some(&PythonSource::Files(vec![
            PythonFile {
                url: "https://files.pythonhosted.org/r/requests_toolbelt-1.0.0-py3-none-any.whl"
                    .to_string(),
                sha256: Some("bbbb".to_string()),
                size: None,
            },
            PythonFile {
                url: "https://files.pythonhosted.org/r/requests-toolbelt-1.0.0.tar.gz".to_string(),
                sha256: Some("aaaa".to_string()),
                size: None,
            },
        ]))
    );
    assert_eq!(
        lock.package("app").map(|package| &package.source),
        Some(&PythonSource::Folder(".".to_string()))
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
            "https://github.com/acme/forked?rev=main#0123456789abcdef".to_string()
        ))
    );
}

#[test]
fn poetry_reads_hashes_git_commits_and_private_indexes() {
    let lock = parse(Lockfile {
        path: "poetry.lock",
        text: POETRY,
    });

    assert_eq!(
        lock.package("PyYAML").map(|package| &package.source),
        Some(&PythonSource::Index {
            hashes: vec!["cccc".to_string()]
        })
    );
    assert_eq!(
        lock.package("forked").map(|package| &package.source),
        Some(&PythonSource::Git(
            "https://github.com/acme/forked.git#0123456789abcdef".to_string()
        ))
    );
    assert_eq!(
        lock.package("inside").map(|package| &package.source),
        Some(&PythonSource::Private(
            "https://pypi.acme.dev/simple".to_string()
        ))
    );
}

#[test]
fn a_uv_lock_written_by_pip_compile_is_read_as_requirements() {
    let lock = parse(Lockfile {
        path: "uv.lock",
        text: "# uv pip compile pyproject.toml -o uv.lock\naiohttp==3.11.18\n    # via app\n",
    });

    assert_eq!(
        lock.package("aiohttp")
            .map(|package| package.version.as_str()),
        Some("3.11.18")
    );
}
