use std::collections::HashMap;

use super::archives::Install;
use super::source::MavenSource;
use super::{JavaEnvironment, Project};
use crate::Error;
use crate::deep::Skip;
use crate::java::gradle::locked_artifacts;
use crate::java::maven::{
    Coordinates, EffectiveRequest, ResolveRequest, effective, parse_pom, resolve,
};

const CENTRAL: &str = "https://repo1.maven.org/maven2";
const GOOGLE: &str = "https://dl.google.com/dl/android/maven2";
const MAVEN_BUILD: &str = "pom.xml";
const GRADLE_LOCKFILE: &str = "gradle.lockfile";
const NO_GRADLE_LOCKFILE: &str = "No gradle.lockfile covers this code. Vigilante does not run Gradle, so it read only the code in the repository.";

impl JavaEnvironment<'_> {
    pub(super) fn load(&mut self, build: String) -> Result<usize, Error> {
        if let Some(project) = self.loaded.get(&build) {
            return Ok(*project);
        }
        let project = self.projects.len();
        self.projects.push(Project {
            classes: HashMap::new(),
        });
        self.loaded.insert(build.clone(), project);
        let text = self
            .tree
            .read(&[&build])?
            .pop()
            .map(|file| file.text)
            .unwrap_or_default();
        let (artifacts, repositories) = if build.ends_with(MAVEN_BUILD) {
            self.maven_artifacts(MavenBuild {
                build: &build,
                text: &text,
            })?
        } else if build.ends_with(GRADLE_LOCKFILE) {
            let mut repositories = self.sources.configured().to_vec();
            repositories.extend([CENTRAL.to_string(), GOOGLE.to_string()]);
            (locked_artifacts(&text), repositories)
        } else {
            self.skip(Skip {
                source: build.clone(),
                reason: NO_GRADLE_LOCKFILE.to_string(),
            });
            (Vec::new(), Vec::new())
        };
        self.install_all(Install {
            project,
            artifacts,
            repositories: &repositories,
            build: &build,
        })?;
        Ok(project)
    }

    fn maven_artifacts(
        &mut self,
        maven: MavenBuild,
    ) -> Result<(Vec<Coordinates>, Vec<String>), Error> {
        let mut local_poms = HashMap::new();
        let manifests: Vec<&str> = self
            .tree
            .paths()
            .iter()
            .map(String::as_str)
            .filter(|path| path.rsplit('/').next() == Some(MAVEN_BUILD))
            .collect();
        for file in self.tree.read(&manifests)? {
            if let Some(pom) = parse_pom(&file.text) {
                let group = pom
                    .group
                    .clone()
                    .or_else(|| {
                        pom.parent
                            .as_ref()
                            .map(|parent| parent.coordinates.group.clone())
                    })
                    .unwrap_or_default();
                self.local_modules
                    .insert((group.clone(), pom.artifact.clone()));
                local_poms.insert((group, pom.artifact), file.path);
            }
        }
        let Some(root) = parse_pom(maven.text) else {
            return Err(Error::new(format!(
                "{} is not a Maven project file",
                maven.build
            )));
        };
        let mut repositories = Vec::new();
        for repository in &root.repositories {
            if self
                .sources
                .allows(&format!("{}/", repository.trim_end_matches('/')))
            {
                repositories.push(repository.clone());
            } else {
                self.skip(Skip {
                    source: maven.build.to_string(),
                    reason: format!("The repository `{repository}` is not public. Add it to `[registries] allow` in vigilante.toml if it needs no credentials."),
                });
            }
        }
        repositories.push(CENTRAL.to_string());
        let mut source = MavenSource {
            tree: self.tree,
            registry: self.registry,
            repositories: repositories.clone(),
            local_poms,
        };
        let root = effective(EffectiveRequest {
            pom: root,
            source: &mut source,
            depth: 0,
        })?;
        self.local_modules.insert((
            root.coordinates.group.clone(),
            root.coordinates.artifact.clone(),
        ));
        let local = self.local_modules.clone();
        let resolution = resolve(ResolveRequest {
            root: &root,
            source: &mut source,
            is_local: &|key| local.contains(key),
        })?;
        for reason in resolution.skipped {
            self.skip(Skip {
                source: maven.build.to_string(),
                reason,
            });
        }
        Ok((resolution.artifacts, repositories))
    }
}

struct MavenBuild<'a> {
    build: &'a str,
    text: &'a str,
}
