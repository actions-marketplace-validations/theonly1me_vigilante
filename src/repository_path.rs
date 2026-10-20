const ROOT: &str = ".";

pub fn parent_folder(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(folder, _)| folder)
}

pub fn top_folder(path: &str) -> &str {
    path.split_once('/').map_or("", |(folder, _)| folder)
}

pub trait Folder {
    fn joined(&self, relative: &str) -> Option<String>;

    fn holds(&self, folder: &str) -> bool;
}

impl Folder for str {
    fn joined(&self, relative: &str) -> Option<String> {
        let mut parts: Vec<&str> = self
            .split('/')
            .filter(|part| !part.is_empty() && *part != ROOT)
            .collect();
        for part in relative.split('/') {
            match part {
                "" | "." => {}
                ".." => {
                    parts.pop()?;
                }
                other => parts.push(other),
            }
        }
        Some(parts.join("/"))
    }

    fn holds(&self, folder: &str) -> bool {
        self == ROOT
            || folder == self
            || folder
                .strip_prefix(self)
                .is_some_and(|rest| rest.starts_with('/'))
    }
}
