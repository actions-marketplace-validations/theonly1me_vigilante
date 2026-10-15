use super::{PythonLock, PythonPackage, PythonSource, normalized, sha256_of};

pub fn parse(text: &str) -> PythonLock {
    let mut lock = PythonLock::default();
    let joined = text.replace("\\\n", " ");
    for line in joined.lines() {
        let line = line.split(" #").next().unwrap_or_default().trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('-') {
            if let Some(url) = line
                .strip_prefix("-e ")
                .map(str::trim)
                .filter(|url| url.starts_with("git+"))
            {
                lock.packages.push(git_requirement(url));
            }
            continue;
        }
        if line.starts_with("git+") {
            lock.packages.push(git_requirement(line));
            continue;
        }
        let mut parts = line.split_whitespace();
        let requirement = parts.next().unwrap_or_default();
        let requirement = requirement.split(';').next().unwrap_or_default();
        let hashes: Vec<String> = parts
            .filter_map(|part| part.strip_prefix("--hash="))
            .filter_map(sha256_of)
            .collect();
        match requirement.split_once("==") {
            Some((name, version)) if !version.contains(['*', ',']) => {
                let name = name.split('[').next().unwrap_or(name);
                lock.packages.push(PythonPackage {
                    name: normalized(name),
                    version: version.to_string(),
                    source: PythonSource::Index { hashes },
                });
            }
            _ => {
                let name: String = requirement
                    .chars()
                    .take_while(|character| {
                        character.is_ascii_alphanumeric() || "-_.".contains(*character)
                    })
                    .collect();
                if !name.is_empty() {
                    lock.unpinned.push(normalized(&name));
                }
            }
        }
    }
    lock
}

fn git_requirement(url: &str) -> PythonPackage {
    let name = url.rsplit_once("#egg=").map_or_else(
        || {
            url.rsplit('/')
                .next()
                .unwrap_or(url)
                .trim_end_matches(".git")
                .to_string()
        },
        |(_, egg)| egg.to_string(),
    );
    let location = url
        .trim_start_matches("git+")
        .split("#egg=")
        .next()
        .unwrap_or(url);
    let (repository, commit) = location.rsplit_once('@').unwrap_or((location, ""));
    PythonPackage {
        name: normalized(&name),
        version: commit.to_string(),
        source: PythonSource::Git(format!("{repository}#{commit}")),
    }
}
