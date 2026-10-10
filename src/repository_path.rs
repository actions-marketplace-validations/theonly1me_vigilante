pub fn parent_folder(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(folder, _)| folder)
}

pub fn joined(folder: &str, relative: &str) -> Option<String> {
    let mut parts: Vec<&str> = folder
        .split('/')
        .filter(|part| !part.is_empty() && *part != ".")
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
