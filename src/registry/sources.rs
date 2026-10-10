use crate::Error;

const PUBLIC_SOURCES: &[&str] = &[
    "https://registry.npmjs.org/",
    "https://registry.yarnpkg.com/",
    "https://codeload.github.com/",
    "https://pypi.org/",
    "https://files.pythonhosted.org/",
];

#[derive(Clone, Debug, Default)]
pub struct Sources {
    allowed: Vec<String>,
}

impl Sources {
    pub fn new(allowed: &[String]) -> Result<Self, Error> {
        let mut prefixes: Vec<String> = PUBLIC_SOURCES
            .iter()
            .map(|source| source.to_string())
            .collect();
        for entry in allowed {
            prefixes.push(normalized(entry)?);
        }
        Ok(Self { allowed: prefixes })
    }

    pub fn allows(&self, url: &str) -> bool {
        self.allowed
            .iter()
            .any(|prefix| url.starts_with(prefix.as_str()))
    }
}

fn normalized(entry: &str) -> Result<String, Error> {
    let invalid = || {
        Error::new(format!(
            "invalid registries.allow entry `{entry}`: use an http or https address without a user name or password"
        ))
    };
    let rest = entry
        .strip_prefix("https://")
        .or_else(|| entry.strip_prefix("http://"))
        .ok_or_else(invalid)?;
    let host = rest.split('/').next().unwrap_or_default();
    if host.is_empty() || host.contains('@') {
        return Err(invalid());
    }
    Ok(if entry.ends_with('/') {
        entry.to_string()
    } else {
        format!("{entry}/")
    })
}
