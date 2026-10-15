use std::collections::HashMap;

use crate::Error;
use crate::java::maven::{Coordinates, Pom, PomSource, parse_pom};
use crate::registry::Registry;
use crate::tree::Tree;

pub struct MavenSource<'a> {
    pub tree: &'a dyn Tree,
    pub registry: &'a Registry,
    pub repositories: Vec<String>,
    pub local_poms: HashMap<(String, String), String>,
}

impl PomSource for MavenSource<'_> {
    fn pom(&mut self, coordinates: &Coordinates) -> Result<Option<Pom>, Error> {
        let key = (coordinates.group.clone(), coordinates.artifact.clone());
        if let Some(path) = self.local_poms.get(&key) {
            let text = self
                .tree
                .read(&[path])?
                .pop()
                .map(|file| file.text)
                .unwrap_or_default();
            return Ok(parse_pom(&text));
        }
        for repository in &self.repositories {
            let url = format!(
                "{}/{}",
                repository.trim_end_matches('/'),
                artifact_path(ArtifactFile {
                    coordinates,
                    extension: "pom",
                })
            );
            if let Some(text) = self.registry.get_text(&url)? {
                return Ok(parse_pom(&text));
            }
        }
        Ok(None)
    }
}

pub struct ArtifactFile<'a> {
    pub coordinates: &'a Coordinates,
    pub extension: &'a str,
}

pub fn artifact_path(file: ArtifactFile) -> String {
    let Coordinates {
        group,
        artifact,
        version,
    } = file.coordinates;
    format!(
        "{}/{artifact}/{version}/{artifact}-{version}.{}",
        group.replace('.', "/"),
        file.extension
    )
}
