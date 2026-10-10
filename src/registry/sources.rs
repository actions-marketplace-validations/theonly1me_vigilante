use crate::Error;

const PUBLIC_SOURCES: &[&str] = &[
    "https://registry.npmjs.org/",
    "https://registry.yarnpkg.com/",
    "https://codeload.github.com/",
    "https://pypi.org/",
    "https://files.pythonhosted.org/",
    "https://repo1.maven.org/maven2/",
    "https://repo.maven.apache.org/maven2/",
    "https://dl.google.com/dl/android/maven2/",
    "https://static.crates.io/",
    "https://api.nuget.org/",
    "https://rubygems.org/",
];

#[derive(Clone, Debug, Default)]
pub struct Sources {
    configured: Vec<String>,
}

impl Sources {
    pub fn new(allowed: &[String]) -> Result<Self, Error> {
        let configured = allowed
            .iter()
            .map(|entry| normalized(entry))
            .collect::<Result<_, _>>()?;
        Ok(Self { configured })
    }

    pub fn allows(&self, url: &str) -> bool {
        PUBLIC_SOURCES
            .iter()
            .copied()
            .chain(self.configured.iter().map(String::as_str))
            .any(|prefix| url.starts_with(prefix))
    }

    pub fn configured(&self) -> &[String] {
        &self.configured
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
