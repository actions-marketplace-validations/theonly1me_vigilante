use super::{Archive, Download};

const GITHUB_HOSTS: &[&str] = &[
    "git+ssh://git@github.com/",
    "git+https://github.com/",
    "git://github.com/",
    "git+http://github.com/",
    "ssh://git@github.com/",
    "https://github.com/",
    "github:",
];

pub struct GitArchive {
    pub download: Download,
    pub commit: String,
}

pub fn github_archive(url: &str) -> Option<GitArchive> {
    let (location, commit) = url.split_once('#')?;
    let commit = commit.trim_start_matches("commit=");
    let is_commit = commit.len() >= 7
        && commit
            .chars()
            .all(|character| character.is_ascii_hexdigit());
    if !is_commit {
        return None;
    }
    let repository = GITHUB_HOSTS
        .iter()
        .find_map(|host| location.strip_prefix(host))?
        .trim_end_matches(".git");
    let (owner, name) = repository.split_once('/')?;
    let is_plain = |part: &str| {
        !part.is_empty()
            && part
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "-_.".contains(character))
    };
    if !is_plain(owner) || !is_plain(name) || name.contains('/') {
        return None;
    }
    Some(GitArchive {
        download: Download {
            url: format!("https://codeload.github.com/{owner}/{name}/tar.gz/{commit}"),
            checksum: None,
            archive: Archive::TarGzip,
        },
        commit: commit.to_string(),
    })
}
