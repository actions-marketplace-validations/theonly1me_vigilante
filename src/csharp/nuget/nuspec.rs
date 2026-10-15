use super::frameworks::{FrameworkChoice, best_framework};
use crate::xml::{Element, parse};

pub struct NuspecQuery<'a> {
    pub text: &'a str,
    pub framework: &'a str,
}

pub fn dependencies_for(query: NuspecQuery) -> Vec<(String, String)> {
    let Some(package) = parse(query.text) else {
        return Vec::new();
    };
    let Some(dependencies) = package
        .child("metadata")
        .and_then(|metadata| metadata.child("dependencies"))
    else {
        return Vec::new();
    };
    let groups = dependencies.all("group");
    if groups.is_empty() {
        return entries(dependencies);
    }
    let frameworks: Vec<&str> = groups
        .iter()
        .filter_map(|group| group.attribute("targetFramework"))
        .collect();
    let best = best_framework(FrameworkChoice {
        project: query.framework,
        candidates: &frameworks,
    });
    groups
        .into_iter()
        .find(|group| match &best {
            Some(best) => group.attribute("targetFramework") == Some(best.as_str()),
            None => group.attribute("targetFramework").is_none(),
        })
        .map(entries)
        .unwrap_or_default()
}

fn entries(group: &Element) -> Vec<(String, String)> {
    group
        .all("dependency")
        .into_iter()
        .filter_map(|dependency| {
            Some((
                dependency.attribute("id")?.trim().to_string(),
                dependency
                    .attribute("version")
                    .unwrap_or_default()
                    .to_string(),
            ))
        })
        .collect()
}
