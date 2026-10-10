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

pub fn pnpm_lockfile(registry: &NpmRegistry, entries: &[PnpmEntry]) -> String {
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

pub fn yarn_lockfile(registry: &NpmRegistry, entries: &[YarnEntry]) -> String {
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
