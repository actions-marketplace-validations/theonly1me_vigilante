use std::collections::{HashSet, VecDeque};

use super::effective::{Effective, EffectiveRequest, PomSource, effective};
use super::pom::{Coordinates, Dependency};
use crate::Error;

const ROOT_SCOPES: &[&str] = &["compile", "runtime", "provided"];
const TRANSITIVE_SCOPES: &[&str] = &["compile", "runtime"];
const DEFAULT_SCOPE: &str = "compile";
const ANY: &str = "*";
const POM_PACKAGING: &str = "pom";

pub struct Resolution {
    pub artifacts: Vec<Coordinates>,
    pub skipped: Vec<String>,
}

pub struct ResolveRequest<'a> {
    pub root: &'a Effective,
    pub source: &'a mut dyn PomSource,
    pub is_local: &'a dyn Fn(&(String, String)) -> bool,
}

pub fn resolve(request: ResolveRequest) -> Result<Resolution, Error> {
    let ResolveRequest {
        root,
        source,
        is_local,
    } = request;
    let mut resolution = Resolution {
        artifacts: Vec::new(),
        skipped: Vec::new(),
    };
    let mut queue: VecDeque<(Dependency, Vec<(String, String)>)> = root
        .dependencies
        .iter()
        .filter(|item| ROOT_SCOPES.contains(&item.scope.as_deref().unwrap_or(DEFAULT_SCOPE)))
        .map(|item| (item.clone(), item.exclusions.clone()))
        .collect();
    let mut seen = HashSet::new();
    while let Some((item, exclusions)) = queue.pop_front() {
        if !seen.insert(item.key()) || is_local(&item.key()) {
            continue;
        }
        if item.classifier.is_some()
            || item
                .kind
                .as_deref()
                .is_some_and(|kind| kind != "jar" && kind != "bundle")
        {
            continue;
        }
        let managed = root
            .management
            .iter()
            .find(|managed| managed.key() == item.key());
        let Some(version) = managed
            .and_then(|managed| managed.version.clone())
            .or(item.version.clone())
        else {
            resolution.skipped.push(format!(
                "`{}:{}` has no version.",
                item.group, item.artifact
            ));
            continue;
        };
        if version.starts_with(['[', '(']) || version.contains("${") {
            resolution.skipped.push(format!(
                "`{}:{}:{version}` has a version that Vigilante cannot read.",
                item.group, item.artifact
            ));
            continue;
        }
        let coordinates = Coordinates {
            group: item.group.clone(),
            artifact: item.artifact.clone(),
            version,
        };
        let Some(pom) = source.pom(&coordinates)? else {
            resolution.skipped.push(format!(
                "`{}:{}:{}` is not in Maven Central.",
                coordinates.group, coordinates.artifact, coordinates.version
            ));
            continue;
        };
        let found = effective(EffectiveRequest {
            pom,
            source,
            depth: 0,
        })?;
        if found.packaging != POM_PACKAGING {
            resolution.artifacts.push(coordinates);
        }
        for child in found.dependencies {
            let scope = child.scope.as_deref().unwrap_or(DEFAULT_SCOPE);
            let is_excluded = exclusions.iter().any(|(group, artifact)| {
                (group == ANY || *group == child.group)
                    && (artifact == ANY || *artifact == child.artifact)
            });
            if child.optional || is_excluded || !TRANSITIVE_SCOPES.contains(&scope) {
                continue;
            }
            let mut inherited = exclusions.clone();
            inherited.extend(child.exclusions.clone());
            queue.push_back((child, inherited));
        }
    }
    Ok(resolution)
}
