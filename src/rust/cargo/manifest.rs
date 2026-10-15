use toml::{Table, Value};

const DEPENDENCY_TABLES: &[&str] = &["dependencies", "build-dependencies", "dev-dependencies"];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Manifest {
    pub name: Option<String>,
    pub library: Option<String>,
    pub library_name: Option<String>,
    pub dependencies: Vec<DeclaredDependency>,
    pub inherited: Vec<String>,
    pub workspace_dependencies: Vec<DeclaredDependency>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeclaredDependency {
    pub local: String,
    pub package: String,
}

pub fn read_manifest(text: &str) -> Option<Manifest> {
    let table: Table = toml::from_str(text).ok()?;
    let text_at = |section: &str, key: &str| {
        table
            .get(section)
            .and_then(|value| value.get(key))
            .and_then(Value::as_str)
            .map(str::to_string)
    };
    let mut sections: Vec<&Table> = DEPENDENCY_TABLES
        .iter()
        .filter_map(|name| table.get(*name).and_then(Value::as_table))
        .collect();
    if let Some(targets) = table.get("target").and_then(Value::as_table) {
        for target in targets.values() {
            sections.extend(
                DEPENDENCY_TABLES
                    .iter()
                    .filter_map(|name| target.get(*name).and_then(Value::as_table)),
            );
        }
    }
    let mut inherited: Vec<String> = sections
        .iter()
        .flat_map(|section| section.iter())
        .filter(|(_, value)| value.get("workspace").and_then(Value::as_bool) == Some(true))
        .map(|(key, _)| key.clone())
        .collect();
    inherited.sort();
    inherited.dedup();
    let workspace = table
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(Value::as_table);
    Some(Manifest {
        name: text_at("package", "name"),
        library: text_at("lib", "path"),
        library_name: text_at("lib", "name"),
        dependencies: declared(&sections),
        inherited,
        workspace_dependencies: declared(&workspace.into_iter().collect::<Vec<_>>()),
    })
}

fn declared(sections: &[&Table]) -> Vec<DeclaredDependency> {
    let mut dependencies: Vec<DeclaredDependency> = sections
        .iter()
        .flat_map(|section| section.iter())
        .map(|(key, value)| DeclaredDependency {
            local: key.replace('-', "_"),
            package: value
                .get("package")
                .and_then(Value::as_str)
                .unwrap_or(key)
                .to_string(),
        })
        .collect();
    dependencies.sort();
    dependencies.dedup();
    dependencies
}
