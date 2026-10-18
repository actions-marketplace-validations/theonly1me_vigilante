use crate::npm_registry::{FakePackage, NpmRegistry};

pub struct PnpmEntry<'a> {
    pub package: &'a FakePackage<'a>,
    pub dependencies: &'a [(&'a str, &'a str)],
    pub is_direct: bool,
}

pub struct YarnEntry<'a> {
    pub package: &'a FakePackage<'a>,
    pub range: &'a str,
    pub dependencies: &'a [(&'a str, &'a str)],
}

pub struct BunEntry<'a> {
    pub key: &'a str,
    pub package: &'a FakePackage<'a>,
    pub dependencies: &'a [(&'a str, &'a str)],
}

pub struct BunLock<'a> {
    pub workspaces: &'a [(&'a str, &'a str)],
    pub entries: &'a [BunEntry<'a>],
}

impl NpmRegistry {
    pub fn bun_lockfile(&self, lock: BunLock) -> String {
        let mut lines = vec![
            "{".to_string(),
            "  \"lockfileVersion\": 1,".to_string(),
            "  \"workspaces\": {".to_string(),
            "    \"\": { \"name\": \"app\", },".to_string(),
        ];
        for (folder, name) in lock.workspaces {
            lines.push(format!("    \"{folder}\": {{ \"name\": \"{name}\", }},"));
        }
        lines.push("  },".to_string());
        lines.push("  \"packages\": {".to_string());
        for (folder, name) in lock.workspaces {
            lines.push(format!("    \"{name}\": [\"{name}@workspace:{folder}\"],"));
        }
        for entry in lock.entries {
            let (url, integrity) = self.tarball(entry.package);
            let dependencies: Vec<String> = entry
                .dependencies
                .iter()
                .map(|(name, range)| format!("\"{name}\": \"{range}\""))
                .collect();
            lines.push(format!(
                "    \"{}\": [\"{}@{}\", \"{url}\", {{ \"dependencies\": {{ {} }} }}, \"{integrity}\"],",
                entry.key,
                entry.package.name,
                entry.package.version,
                dependencies.join(", ")
            ));
        }
        lines.push("  }".to_string());
        lines.push("}".to_string());
        lines.join("\n") + "\n"
    }

    pub fn pnpm_lockfile(&self, entries: &[PnpmEntry]) -> String {
        pnpm_lockfile(Written {
            registry: self,
            entries,
        })
    }

    pub fn yarn_lockfile(&self, entries: &[YarnEntry]) -> String {
        yarn_lockfile(Written {
            registry: self,
            entries,
        })
    }
}

struct Written<'a, T> {
    registry: &'a NpmRegistry,
    entries: &'a [T],
}

fn pnpm_lockfile(written: Written<PnpmEntry>) -> String {
    let Written { registry, entries } = written;
    let mut lines = vec![
        "lockfileVersion: '9.0'".to_string(),
        String::new(),
        "importers:".to_string(),
        "  .:".to_string(),
        "    dependencies:".to_string(),
    ];
    for entry in entries.iter().filter(|entry| entry.is_direct) {
        lines.push(format!("      {}:", entry.package.name));
        lines.push(format!("        specifier: ^{}", entry.package.version));
        lines.push(format!("        version: {}", entry.package.version));
    }
    lines.push("packages:".to_string());
    for entry in entries {
        let (url, integrity) = registry.tarball(entry.package);
        lines.push(format!(
            "  {}@{}:",
            entry.package.name, entry.package.version
        ));
        lines.push(format!(
            "    resolution: {{integrity: {integrity}, tarball: '{url}'}}"
        ));
    }
    lines.push("snapshots:".to_string());
    for entry in entries {
        lines.push(format!(
            "  {}@{}:",
            entry.package.name, entry.package.version
        ));
        if !entry.dependencies.is_empty() {
            lines.push("    dependencies:".to_string());
            lines.extend(
                entry
                    .dependencies
                    .iter()
                    .map(|(name, version)| format!("      {name}: {version}")),
            );
        }
    }
    lines.join("\n") + "\n"
}

fn yarn_lockfile(written: Written<YarnEntry>) -> String {
    let Written { registry, entries } = written;
    let mut lines = vec!["# yarn lockfile v1".to_string(), String::new()];
    for entry in entries {
        let (url, integrity) = registry.tarball(entry.package);
        lines.push(String::new());
        lines.push(format!("{}@{}:", entry.package.name, entry.range));
        lines.push(format!("  version \"{}\"", entry.package.version));
        lines.push(format!("  resolved \"{url}\""));
        lines.push(format!("  integrity {integrity}"));
        if !entry.dependencies.is_empty() {
            lines.push("  dependencies:".to_string());
            lines.extend(
                entry
                    .dependencies
                    .iter()
                    .map(|(name, range)| format!("    {name} \"{range}\"")),
            );
        }
    }
    lines.join("\n") + "\n"
}
