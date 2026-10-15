use std::collections::HashMap;

use super::feeds::{FeedInput, PackageAddress, content_checksum, feeds};
use super::install::{Install, Planned};
use super::parallel::{ParallelWork, each_in_parallel};
use super::resolution::{ResolveInput, resolve};
use super::{CSharpEnvironment, Search};
use crate::Error;
use crate::csharp::nuget::lockfile::{LockedKind, read_lockfile};
use crate::csharp::nuget::project::{ProjectInput, read_project};
use crate::deep::Skip;
use crate::repository_path::{Folder, parent_folder};

const DEFAULT_FRAMEWORK: &str = "net48";
const LOCKFILE: &str = "packages.lock.json";
const BUILD_PROPERTIES: &[&str] = &["Directory.Build.props"];
const CENTRAL_VERSIONS: &[&str] = &["Directory.Packages.props"];
const CONFIGURATIONS: &[&str] = &["nuget.config", "NuGet.config", "NuGet.Config"];

impl CSharpEnvironment<'_> {
    pub(super) fn load(&mut self, project_path: &str) -> Result<usize, Error> {
        let position = self.project_types.len();
        self.project_types.push(HashMap::new());
        self.project_of_file
            .insert(project_path.to_string(), position);
        let folder = parent_folder(project_path).to_string();
        let read = |path: Option<String>| -> Result<Option<String>, Error> {
            let Some(path) = path else {
                return Ok(None);
            };
            Ok(self.tree.read(&[&path])?.pop().map(|file| file.text))
        };
        let project_text = read(Some(project_path.to_string()))?.unwrap_or_default();
        let build = read(self.nearest(Search {
            start: &folder,
            names: BUILD_PROPERTIES,
        }))?;
        let central = read(self.nearest(Search {
            start: &folder,
            names: CENTRAL_VERSIONS,
        }))?;
        let configuration_path = self.nearest(Search {
            start: &folder,
            names: CONFIGURATIONS,
        });
        let configuration = read(configuration_path.clone())?;
        let lockfile = read(
            folder
                .joined(LOCKFILE)
                .filter(|path| self.tree.paths().contains(path)),
        )?;
        let project = read_project(ProjectInput {
            project: &project_text,
            build_properties: build.as_deref(),
            central_versions: central.as_deref(),
        })
        .unwrap_or_default();
        let framework = project
            .frameworks
            .first()
            .cloned()
            .unwrap_or_else(|| DEFAULT_FRAMEWORK.to_string());
        let found = feeds(FeedInput {
            registry: self.registry,
            sources: self.sources,
            config: configuration.as_deref(),
        })?;
        for address in &found.not_public {
            self.skip(Skip {
                source: configuration_path
                    .clone()
                    .unwrap_or_else(|| project_path.to_string()),
                reason: format!("The registry `{address}` is not public. Add it to `[registries] allow` in vigilante.toml if it needs no credentials."),
            });
        }
        let planned = match lockfile {
            Some(text) => read_lockfile(&text)?
                .into_iter()
                .filter(|locked| locked.kind == LockedKind::Package)
                .filter_map(|locked| {
                    Some(Planned {
                        checksum: locked.content_hash.as_deref().and_then(content_checksum),
                        version: locked.version?,
                        id: locked.id,
                        feed: None,
                    })
                })
                .collect(),
            None => {
                let resolution = resolve(ResolveInput {
                    references: &project.references,
                    framework: &framework,
                    feeds: &found.feeds,
                    registry: self.registry,
                })?;
                for reason in resolution.skipped {
                    self.skip(Skip {
                        source: project_path.to_string(),
                        reason,
                    });
                }
                let registry = self.registry;
                let feeds = &found.feeds;
                let checksums = each_in_parallel(ParallelWork {
                    items: &resolution.packages,
                    work: |resolved: &super::resolution::Resolved| {
                        let feed = &feeds[resolved.feed?];
                        PackageAddress {
                            feed,
                            id: &resolved.id,
                            version: &resolved.version,
                        }
                        .checksum(registry)
                        .ok()?
                    },
                });
                resolution
                    .packages
                    .into_iter()
                    .zip(checksums)
                    .map(|(resolved, checksum)| Planned {
                        id: resolved.id,
                        version: resolved.version,
                        checksum,
                        feed: resolved.feed,
                    })
                    .collect()
            }
        };
        let installed = self.install_all(Install {
            project: position,
            project_path,
            planned,
            feeds: &found.feeds,
            framework: &framework,
        })?;
        let mut types = HashMap::new();
        for package in installed {
            for (path, (assembly, index)) in &self.packages[package].files {
                let definition = &self.packages[package].assemblies[*assembly].types[*index];
                let mut key = definition.namespace.clone();
                for part in definition.enclosing.iter().chain([&definition.name]) {
                    if !key.is_empty() {
                        key.push('.');
                    }
                    key.push_str(part);
                }
                types.entry(key).or_insert_with(|| (package, path.clone()));
            }
        }
        self.project_types[position] = types;
        Ok(position)
    }
}
