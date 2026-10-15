use std::collections::HashMap;

use crate::xml::{Element, parse};

pub struct ProjectInput<'a> {
    pub project: &'a str,
    pub build_properties: Option<&'a str>,
    pub central_versions: Option<&'a str>,
}

#[derive(Debug, Default)]
pub struct Project {
    pub frameworks: Vec<String>,
    pub references: Vec<(String, String)>,
}

pub fn read_project(input: ProjectInput) -> Option<Project> {
    let project = parse(input.project)?;
    let central_file = input.central_versions.and_then(parse);
    let mut properties = input
        .build_properties
        .and_then(parse)
        .map(|build| properties_of(&build))
        .unwrap_or_default();
    if let Some(central_file) = &central_file {
        properties.extend(properties_of(central_file));
    }
    properties.extend(properties_of(&project));
    let expand = |text: &str| {
        substitute(Substitution {
            text,
            properties: &properties,
            depth: 0,
        })
    };
    let central: HashMap<String, String> = central_file
        .as_ref()
        .map(|central| {
            central
                .descendants("PackageVersion")
                .into_iter()
                .filter_map(|item| {
                    Some((
                        item.attribute("Include")?.trim().to_lowercase(),
                        item.value("Version")?,
                    ))
                })
                .collect()
        })
        .unwrap_or_default();
    let frameworks = properties
        .get("TargetFrameworks")
        .or_else(|| properties.get("TargetFramework"))
        .map(|text| {
            expand(text)
                .split(';')
                .map(str::trim)
                .filter(|framework| !framework.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    let references = project
        .descendants("PackageReference")
        .into_iter()
        .filter_map(|item| {
            let id = item.attribute("Include")?.trim().to_string();
            let version = item
                .value("VersionOverride")
                .or_else(|| item.value("Version"))
                .or_else(|| central.get(&id.to_lowercase()).cloned())
                .unwrap_or_default();
            Some((id, expand(&version)))
        })
        .collect();
    Some(Project {
        frameworks,
        references,
    })
}

fn properties_of(root: &Element) -> HashMap<String, String> {
    root.all("PropertyGroup")
        .into_iter()
        .flat_map(|group| &group.children)
        .map(|property| (property.name.clone(), property.text.trim().to_string()))
        .collect()
}

const MAXIMUM_DEPTH: usize = 8;

struct Substitution<'a> {
    text: &'a str,
    properties: &'a HashMap<String, String>,
    depth: usize,
}

fn substitute(substitution: Substitution) -> String {
    let Substitution {
        text,
        properties,
        depth,
    } = substitution;
    if depth > MAXIMUM_DEPTH {
        return text.to_string();
    }
    let mut result = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("$(") {
        let Some(length) = rest[start..].find(')') else {
            break;
        };
        result.push_str(&rest[..start]);
        let key = &rest[start + 2..start + length];
        if let Some(value) = properties.get(key) {
            result.push_str(&substitute(Substitution {
                text: value,
                properties,
                depth: depth + 1,
            }));
        }
        rest = &rest[start + length + 1..];
    }
    result.push_str(rest);
    result
}
