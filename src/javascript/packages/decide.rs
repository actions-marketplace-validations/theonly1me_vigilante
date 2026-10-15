use super::{Locate, Located, Packages};
use crate::Error;
use crate::javascript::paths::is_code_file;
use crate::lockfiles::{Found, LockedPackage, Lookup, PackageSource, Place};
use crate::registry::{
    Archive, Checksum, Download, FetchRequest, Fetched, PackageId, Prefetch, Registry,
    github_archive,
};
use crate::repository_path::Folder;

const ECOSYSTEM: &str = "npm";
const MANIFEST: &str = "package.json";

pub enum Decision {
    Located(Located),
    NotInLockfile,
    Skip { key: String, reason: String },
    Fetch(Planned),
}

pub struct Planned {
    pub key: String,
    pub package: LockedPackage,
    pub download: Download,
    pub commit: Option<String>,
}

impl Planned {
    pub fn label(&self) -> String {
        format!("`{}@{}`", self.package.name, self.package.version)
    }

    fn prefetch(&self) -> Prefetch {
        Prefetch {
            ecosystem: ECOSYSTEM,
            name: self.package.name.clone(),
            version: self
                .commit
                .clone()
                .unwrap_or_else(|| self.package.version.clone()),
            download: self.download.clone(),
            keep: is_kept,
        }
    }

    pub fn fetch_from(&self, registry: &Registry) -> Result<Fetched, Error> {
        registry.fetch(FetchRequest {
            package: PackageId {
                ecosystem: ECOSYSTEM,
                name: &self.package.name,
                version: self.commit.as_deref().unwrap_or(&self.package.version),
            },
            download: &self.download,
            keep: &is_kept,
        })
    }
}

impl Packages<'_> {
    pub(super) fn decide(&self, request: &Locate) -> Decision {
        let project = self.projects.project(request.project);
        let place = if request.is_from_repository {
            Place::Repository {
                folder: request.folder,
            }
        } else {
            Place::Package {
                key: request.folder,
            }
        };
        let lookup = Lookup {
            place,
            name: request.name,
            declared: request.declared,
        };
        let Some(Found { key, package }) = project.lockfile.find(lookup) else {
            return Decision::NotInLockfile;
        };
        if let Some(known) = self.installed_at.get(&(request.project, key.clone())) {
            return Decision::Located(known.map_or(Located::Missing, Located::Package));
        }
        let label = format!("`{}@{}`", package.name, package.version);
        let skip = |reason: String| Decision::Skip {
            key: key.clone(),
            reason,
        };
        let (download, commit) = match &package.source {
            PackageSource::Folder(folder) => {
                return Decision::Located(
                    project
                        .folder
                        .joined(folder)
                        .map_or(Located::Missing, Located::Repository),
                );
            }
            PackageSource::Registry { url, integrity } if self.sources.allows(url) => (
                Download {
                    url: url.clone(),
                    checksum: integrity.as_deref().and_then(Checksum::from_integrity),
                    archive: Archive::TarGzip,
                },
                None,
            ),
            PackageSource::Registry { url, .. } => {
                return skip(format!(
                    "{label} comes from `{}`, which is not a public registry. Add it to `[registries] allow` in vigilante.toml if it needs no credentials.",
                    origin(url)
                ));
            }
            PackageSource::Git(url) => match github_archive(url) {
                Some(archive) => (archive.download, Some(archive.commit)),
                None => {
                    return skip(format!(
                        "{label} comes from a git repository outside GitHub (`{url}`)."
                    ));
                }
            },
            PackageSource::Bundled => {
                return skip(format!("{label} is bundled inside another package."));
            }
            PackageSource::Unsupported(source) => {
                return skip(format!(
                    "{label} comes from `{source}`, which Vigilante cannot read."
                ));
            }
        };
        Decision::Fetch(Planned {
            key,
            package,
            download,
            commit,
        })
    }

    pub fn prefetch(&self, requests: &[Locate]) {
        let mut planned: Vec<Prefetch> = requests
            .iter()
            .filter_map(|request| match self.decide(request) {
                Decision::Fetch(planned) => Some(planned.prefetch()),
                _ => None,
            })
            .collect();
        planned.sort_by(|left, right| left.download.url.cmp(&right.download.url));
        planned.dedup_by(|left, right| left.download.url == right.download.url);
        self.registry.prefetch(&planned);
    }
}

fn origin(url: &str) -> String {
    let after_scheme = url.split_once("://").map_or(url, |(_, rest)| rest);
    let host = after_scheme.split('/').next().unwrap_or(after_scheme);
    let scheme = url.split_once("://").map_or("https", |(scheme, _)| scheme);
    format!("{scheme}://{}", host.rsplit('@').next().unwrap_or(host))
}

fn is_kept(path: &str) -> bool {
    is_code_file(path) || path.ends_with(MANIFEST)
}
