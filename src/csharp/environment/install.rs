use std::collections::{BTreeSet, HashMap};
use std::fs;

use super::feeds::{Feed, PackageAddress};
use super::{CSharpEnvironment, Package};
use crate::Error;
use crate::csharp::metadata::read;
use crate::csharp::nuget::frameworks::{FrameworkChoice, best_framework};
use crate::deep::Skip;
use crate::registry::{
    Archive, Checksum, Download, FetchRequest, Fetched, PackageId, Prefetch, folder_files,
};

const ECOSYSTEM: &str = "nuget";
const LIBRARIES: &str = "lib/";
const ASSEMBLY_SUFFIX: &str = ".dll";

pub struct Planned {
    pub id: String,
    pub version: String,
    pub checksum: Option<Checksum>,
    pub feed: Option<usize>,
}

pub struct Install<'a> {
    pub project: usize,
    pub project_path: &'a str,
    pub planned: Vec<Planned>,
    pub feeds: &'a [Feed],
    pub framework: &'a str,
}

struct Source<'a> {
    item: &'a Planned,
    feed: &'a Feed,
}

fn nupkg(source: Source) -> Download {
    let Source { item, feed } = source;
    Download {
        url: PackageAddress {
            feed,
            id: &item.id,
            version: &item.version,
        }
        .file("nupkg"),
        checksum: item.checksum.clone(),
        archive: Archive::Zip,
    }
}

fn is_kept(path: &str) -> bool {
    path.starts_with(LIBRARIES) && path.to_lowercase().ends_with(ASSEMBLY_SUFFIX)
}

impl CSharpEnvironment<'_> {
    pub(super) fn install_all(&mut self, install: Install) -> Result<Vec<usize>, Error> {
        let Install {
            project,
            project_path,
            planned,
            feeds,
            framework,
        } = install;

        let prefetches: Vec<Prefetch> = planned
            .iter()
            .filter(|item| item.checksum.is_some())
            .filter_map(|item| {
                Some(Prefetch {
                    ecosystem: ECOSYSTEM,
                    name: item.id.to_lowercase(),
                    version: item.version.clone(),
                    download: nupkg(Source {
                        item,
                        feed: feeds.get(item.feed.unwrap_or(0))?,
                    }),
                    keep: is_kept,
                })
            })
            .collect();
        self.registry.prefetch(&prefetches);
        let mut installed = Vec::new();
        for item in planned {
            let label = format!("`{} {}`", item.id, item.version);
            let key = (item.id.to_lowercase(), item.version.clone());
            if let Some(known) = self.package_of.get(&key) {
                installed.extend(*known);
                continue;
            }
            if item.checksum.is_none() {
                self.skip(Skip {
                    source: project_path.to_string(),
                    reason: format!("{label} has no checksum, so Vigilante did not read it."),
                });
                self.package_of.insert(key, None);
                continue;
            }
            let candidates: Vec<&Feed> = match item.feed {
                Some(feed) => feeds.get(feed).into_iter().collect(),
                None => feeds.iter().collect(),
            };
            let mut folder = None;
            for feed in candidates {
                let request = nupkg(Source { item: &item, feed });
                if let Fetched::Folder(found) = self.registry.fetch(FetchRequest {
                    package: PackageId {
                        ecosystem: ECOSYSTEM,
                        name: &key.0,
                        version: &item.version,
                    },
                    download: &request,
                    keep: &is_kept,
                })? {
                    folder = Some(found);
                    break;
                }
            }
            let Some(folder) = folder else {
                self.skip(Skip {
                    source: project_path.to_string(),
                    reason: format!("{label} is not in any feed that Vigilante can read."),
                });
                self.package_of.insert(key, None);
                continue;
            };
            let files = folder_files(&folder)?;
            let frameworks: BTreeSet<&str> = files
                .iter()
                .filter_map(|path| path.strip_prefix(LIBRARIES)?.split('/').next())
                .collect();
            let candidates: Vec<&str> = frameworks.into_iter().collect();
            let chosen = best_framework(FrameworkChoice {
                project: framework,
                candidates: &candidates,
            });
            let mut package = Package {
                id: item.id.clone(),
                project,
                assemblies: Vec::new(),
                files: HashMap::new(),
            };
            if let Some(chosen) = chosen {
                let prefix = format!("{LIBRARIES}{chosen}/");
                for path in files.iter().filter(|path| {
                    path.strip_prefix(&prefix)
                        .is_some_and(|rest| !rest.contains('/'))
                }) {
                    let Some(assembly) = fs::read(folder.join(path))
                        .ok()
                        .and_then(|bytes| read(&bytes))
                    else {
                        continue;
                    };
                    let position = package.assemblies.len();
                    for (index, definition) in assembly.types.iter().enumerate() {
                        let enclosing = definition
                            .enclosing
                            .as_ref()
                            .map(|outer| format!("{outer}+"))
                            .unwrap_or_default();
                        package.files.insert(
                            format!("{}/{enclosing}{}.cs", definition.namespace, definition.name),
                            (position, index),
                        );
                    }
                    package.assemblies.push(assembly);
                }
            }
            self.packages.push(package);
            let index = self.packages.len() - 1;
            self.package_of.insert(key, Some(index));
            installed.push(index);
        }
        Ok(installed)
    }
}
