use serde_json::Value;

use crate::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockedKind {
    Package,
    Project,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockedPackage {
    pub id: String,
    pub version: Option<String>,
    pub content_hash: Option<String>,
    pub kind: LockedKind,
}

pub fn read_lockfile(text: &str) -> Result<Vec<LockedPackage>, Error> {
    let document: Value = serde_json::from_str(text)
        .map_err(|error| Error::new(format!("packages.lock.json: {error}")))?;
    let Some(frameworks) = document.get("dependencies").and_then(Value::as_object) else {
        return Ok(Vec::new());
    };
    let Some(packages) = frameworks
        .iter()
        .find(|(framework, _)| !framework.contains('/'))
        .and_then(|(_, packages)| packages.as_object())
    else {
        return Ok(Vec::new());
    };
    let text_at =
        |entry: &Value, key: &str| entry.get(key).and_then(Value::as_str).map(str::to_string);
    let mut locked: Vec<LockedPackage> = packages
        .iter()
        .map(|(id, entry)| LockedPackage {
            id: id.clone(),
            version: text_at(entry, "resolved"),
            content_hash: text_at(entry, "contentHash"),
            kind: if entry.get("type").and_then(Value::as_str) == Some("Project") {
                LockedKind::Project
            } else {
                LockedKind::Package
            },
        })
        .collect();
    locked.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(locked)
}
