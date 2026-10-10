use std::collections::BTreeMap;

use super::xml::{Element, parse};

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Coordinates {
    pub group: String,
    pub artifact: String,
    pub version: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Dependency {
    pub group: String,
    pub artifact: String,
    pub version: Option<String>,
    pub scope: Option<String>,
    pub kind: Option<String>,
    pub classifier: Option<String>,
    pub optional: bool,
    pub exclusions: Vec<(String, String)>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ParentReference {
    pub coordinates: Coordinates,
    pub relative_path: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pom {
    pub group: Option<String>,
    pub artifact: String,
    pub version: Option<String>,
    pub packaging: Option<String>,
    pub parent: Option<ParentReference>,
    pub properties: BTreeMap<String, String>,
    pub management: Vec<Dependency>,
    pub dependencies: Vec<Dependency>,
    pub repositories: Vec<String>,
}

impl Dependency {
    pub fn key(&self) -> (String, String) {
        (self.group.clone(), self.artifact.clone())
    }
}

pub fn parse_pom(text: &str) -> Option<Pom> {
    let project = parse(text)?;
    if project.name != "project" {
        return None;
    }
    let parent = project.child("parent").map(|parent| ParentReference {
        coordinates: Coordinates {
            group: parent.text_of("groupId").unwrap_or_default(),
            artifact: parent.text_of("artifactId").unwrap_or_default(),
            version: parent.text_of("version").unwrap_or_default(),
        },
        relative_path: parent.text_of("relativePath"),
    });
    Some(Pom {
        group: project.text_of("groupId"),
        artifact: project.text_of("artifactId").unwrap_or_default(),
        version: project.text_of("version"),
        packaging: project.text_of("packaging"),
        parent,
        properties: project
            .child("properties")
            .map(|properties| {
                properties
                    .children
                    .iter()
                    .map(|property| (property.name.clone(), property.text.trim().to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        management: project
            .child("dependencyManagement")
            .map(dependencies_of)
            .unwrap_or_default(),
        dependencies: dependencies_of(&project),
        repositories: project
            .child("repositories")
            .map(|list| list.all("repository"))
            .unwrap_or_default()
            .into_iter()
            .filter_map(|repository| repository.text_of("url"))
            .collect(),
    })
}

fn dependencies_of(holder: &Element) -> Vec<Dependency> {
    holder
        .child("dependencies")
        .map(|list| list.all("dependency"))
        .unwrap_or_default()
        .into_iter()
        .map(|dependency| Dependency {
            group: dependency.text_of("groupId").unwrap_or_default(),
            artifact: dependency.text_of("artifactId").unwrap_or_default(),
            version: dependency.text_of("version"),
            scope: dependency.text_of("scope"),
            kind: dependency.text_of("type"),
            classifier: dependency.text_of("classifier"),
            optional: dependency.text_of("optional").as_deref() == Some("true"),
            exclusions: dependency
                .child("exclusions")
                .map(|exclusions| exclusions.all("exclusion"))
                .unwrap_or_default()
                .into_iter()
                .map(|exclusion| {
                    (
                        exclusion.text_of("groupId").unwrap_or_default(),
                        exclusion.text_of("artifactId").unwrap_or_default(),
                    )
                })
                .collect(),
        })
        .collect()
}
