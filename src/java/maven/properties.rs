use std::collections::BTreeMap;

use super::pom::{Dependency, Pom};

const MAXIMUM_PASSES: usize = 8;

pub struct Properties(BTreeMap<String, String>);

pub struct PropertiesInput<'a> {
    pub chain: &'a [Pom],
    pub group: &'a str,
    pub version: &'a str,
}

impl Properties {
    pub fn new(input: PropertiesInput) -> Self {
        let PropertiesInput {
            chain,
            group,
            version,
        } = input;
        let mut values = BTreeMap::new();
        for pom in chain.iter().rev() {
            values.extend(pom.properties.clone());
        }
        let Some(child) = chain.first() else {
            return Self(values);
        };
        for (key, value) in [
            ("project.groupId", group),
            ("project.artifactId", &child.artifact),
            ("project.version", version),
            ("pom.version", version),
            ("version", version),
            ("groupId", group),
        ] {
            values.insert(key.to_string(), value.to_string());
        }
        if let Some(parent) = child.parent.as_ref().map(|parent| &parent.coordinates) {
            values.insert("project.parent.version".to_string(), parent.version.clone());
            values.insert("project.parent.groupId".to_string(), parent.group.clone());
            values.insert("parent.version".to_string(), parent.version.clone());
        }
        Self(values)
    }

    pub fn interpolate(&self, text: &str) -> String {
        let mut current = text.to_string();
        for _ in 0..MAXIMUM_PASSES {
            let Some(start) = current.find("${") else {
                break;
            };
            let Some(length) = current[start..].find('}') else {
                break;
            };
            let key = &current[start + 2..start + length];
            let Some(value) = self.0.get(key) else {
                break;
            };
            current = format!(
                "{}{value}{}",
                &current[..start],
                &current[start + length + 1..]
            );
        }
        current
    }

    pub fn dependency(&self, dependency: &Dependency) -> Dependency {
        let text = |value: &Option<String>| value.as_deref().map(|value| self.interpolate(value));
        Dependency {
            group: self.interpolate(&dependency.group),
            artifact: self.interpolate(&dependency.artifact),
            version: text(&dependency.version),
            scope: text(&dependency.scope),
            kind: text(&dependency.kind),
            classifier: text(&dependency.classifier),
            optional: dependency.optional,
            exclusions: dependency.exclusions.clone(),
        }
    }
}
