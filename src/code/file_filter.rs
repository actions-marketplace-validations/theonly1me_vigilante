use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::Serialize;
use serde_json::Value;

use crate::Error;

pub struct FileFilter {
    files: Option<GlobSet>,
    ignores: Option<GlobSet>,
}

pub struct RuleGlobs<'a, T> {
    pub files: &'a Option<Vec<T>>,
    pub ignores: &'a Option<Vec<T>>,
}

impl FileFilter {
    pub fn new<T: Serialize>(globs: RuleGlobs<T>) -> Result<Self, Error> {
        Ok(Self {
            files: glob_set(globs.files)?,
            ignores: glob_set(globs.ignores)?,
        })
    }

    pub fn allows(&self, path: &str) -> bool {
        let is_ignored = self
            .ignores
            .as_ref()
            .is_some_and(|ignores| ignores.is_match(path));
        let is_listed = self.files.as_ref().is_none_or(|files| files.is_match(path));
        is_listed && !is_ignored
    }
}

fn glob_set<T: Serialize>(globs: &Option<Vec<T>>) -> Result<Option<GlobSet>, Error> {
    let Some(globs) = globs else {
        return Ok(None);
    };
    let mut builder = GlobSetBuilder::new();
    for glob in globs {
        let value = serde_json::to_value(glob).map_err(|error| Error::new(error.to_string()))?;
        let (pattern, case_insensitive) = match &value {
            Value::String(pattern) => (pattern.as_str(), false),
            Value::Object(fields) => (
                fields
                    .get("glob")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
                fields
                    .get("caseInsensitive")
                    .and_then(Value::as_bool)
                    .unwrap_or_default(),
            ),
            _ => return Err(Error::new(format!("invalid file glob {value}"))),
        };
        let glob = GlobBuilder::new(pattern)
            .case_insensitive(case_insensitive)
            .build()
            .map_err(|error| Error::new(format!("invalid file glob `{pattern}`: {error}")))?;
        builder.add(glob);
    }
    builder
        .build()
        .map(Some)
        .map_err(|error| Error::new(error.to_string()))
}
