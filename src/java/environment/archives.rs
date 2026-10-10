use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use super::source::{ArtifactFile, artifact_path};
use super::{Archive, JavaEnvironment};
use crate::Error;
use crate::deep::Skip;
use crate::java::maven::Coordinates;
use crate::registry::{
    Archive as Packed, Checksum, Download, FetchRequest, Fetched, PackageId, Prefetch, Registry,
    folder_files,
};

const ECOSYSTEM: &str = "maven";
const MAXIMUM_LOOKUPS_AT_ONCE: usize = 16;

pub struct Install<'a> {
    pub project: usize,
    pub artifacts: Vec<Coordinates>,
    pub repositories: &'a [String],
    pub build: &'a str,
}

struct Located {
    coordinates: Coordinates,
    download: Option<Download>,
}

impl JavaEnvironment<'_> {
    pub(super) fn install_all(&mut self, install: Install) -> Result<(), Error> {
        let located = locate_all(Locating {
            registry: self.registry,
            artifacts: &install.artifacts,
            repositories: install.repositories,
        });
        let planned: Vec<Prefetch> = located
            .iter()
            .filter_map(|item| {
                Some(Prefetch {
                    ecosystem: ECOSYSTEM,
                    name: format!("{}:{}", item.coordinates.group, item.coordinates.artifact),
                    version: item.coordinates.version.clone(),
                    download: item.download.clone()?,
                    keep: is_kept,
                })
            })
            .collect();
        self.registry.prefetch(&planned);
        for item in located {
            let name = format!("{}:{}", item.coordinates.group, item.coordinates.artifact);
            let label = format!("`{name}:{}`", item.coordinates.version);
            let Some(download) = item.download else {
                self.skip(Skip {
                    source: install.build.to_string(),
                    reason: format!("{label} is not in any repository that Vigilante can read."),
                });
                continue;
            };
            let fetched = self.registry.fetch(FetchRequest {
                package: PackageId {
                    ecosystem: ECOSYSTEM,
                    name: &name,
                    version: &item.coordinates.version,
                },
                download: &download,
                keep: &is_kept,
            })?;
            let folder = match fetched {
                Fetched::Folder(folder) => folder,
                Fetched::NotFound | Fetched::TooLarge => {
                    self.skip(Skip {
                        source: install.build.to_string(),
                        reason: format!("{label} could not be read from `{}`.", download.url),
                    });
                    continue;
                }
            };
            let files = folder_files(&folder)?;
            let index = self.archives.len();
            let classes = &mut self.projects[install.project].classes;
            for file in &files {
                classes.entry(file.clone()).or_insert(index);
            }
            self.archives.push(Archive {
                name,
                folder,
                files,
                project: install.project,
            });
        }
        Ok(())
    }
}

struct Locating<'a> {
    registry: &'a Registry,
    artifacts: &'a [Coordinates],
    repositories: &'a [String],
}

fn locate_all(locating: Locating) -> Vec<Located> {
    let next = AtomicUsize::new(0);
    let workers = MAXIMUM_LOOKUPS_AT_ONCE.min(locating.artifacts.len());
    let mut found: Vec<(usize, Located)> = thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                scope.spawn(|| {
                    let mut done = Vec::new();
                    while let Some(position) = Some(next.fetch_add(1, Ordering::Relaxed))
                        .filter(|position| *position < locating.artifacts.len())
                    {
                        let coordinates = &locating.artifacts[position];
                        done.push((
                            position,
                            locate(Locate {
                                registry: locating.registry,
                                coordinates,
                                repositories: locating.repositories,
                            }),
                        ));
                    }
                    done
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().unwrap_or_default())
            .collect()
    });
    found.sort_by_key(|(position, _)| *position);
    found.into_iter().map(|(_, located)| located).collect()
}

struct Locate<'a> {
    registry: &'a Registry,
    coordinates: &'a Coordinates,
    repositories: &'a [String],
}

fn locate(request: Locate) -> Located {
    let path = artifact_path(ArtifactFile {
        coordinates: request.coordinates,
        extension: "jar",
    });
    let download = request.repositories.iter().find_map(|repository| {
        let url = format!("{}/{path}", repository.trim_end_matches('/'));
        let checksum = request.registry.get_text(&format!("{url}.sha1")).ok()??;
        Some(Download {
            checksum: Checksum::sha1_hex(&checksum),
            url,
            archive: Packed::Zip,
        })
    });
    Located {
        coordinates: request.coordinates.clone(),
        download,
    }
}

fn is_kept(path: &str) -> bool {
    path.ends_with(".class")
        && !path.starts_with("META-INF/")
        && !path.ends_with("module-info.class")
}
