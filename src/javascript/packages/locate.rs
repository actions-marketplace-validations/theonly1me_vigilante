use super::{Installed, Locate, Located, Packages};
use crate::Error;
use crate::javascript::files::{folder_files, package_root};
use crate::javascript::manifest::Manifest;
use crate::javascript::paths::{install_candidates, is_code_file, joined};
use crate::lockfiles::PackageSource;
use crate::registry::{
    Archive, Checksum, Download, FetchRequest, Fetched, PackageId, github_archive,
};

const ECOSYSTEM: &str = "npm";
const MANIFEST: &str = "package.json";

impl Packages<'_> {
    pub fn locate(&mut self, request: Locate) -> Result<Located, Error> {
        let project = self.projects.project(request.project);
        let lockfile_path = project.file.clone();
        let found = install_candidates(request.folder, request.name)
            .into_iter()
            .find_map(|key| Some((project.lockfile.package(&key)?.clone(), key)));
        let Some((package, key)) = found else {
            if request.is_from_repository {
                self.skip(
                    &lockfile_path,
                    format!("`{}` is not in the lockfile.", request.name),
                );
            }
            return Ok(Located::Missing);
        };
        let project_folder = project.folder.clone();
        if let Some(known) = self.installed_at.get(&(request.project, key.clone())) {
            return Ok(known.map_or(Located::Missing, Located::Package));
        }
        let label = format!("`{}@{}`", package.name, package.version);
        let mut commit = None;
        let download = match &package.source {
            PackageSource::Folder(folder) => {
                return Ok(
                    joined(&project_folder, folder).map_or(Located::Missing, Located::Repository)
                );
            }
            PackageSource::Registry { url, integrity } if self.sources.allows(url) => Download {
                url: url.clone(),
                checksum: integrity.as_deref().and_then(Checksum::from_integrity),
                archive: Archive::TarGzip,
            },
            PackageSource::Registry { url, .. } => {
                let reason = format!(
                    "{label} comes from `{}`, which is not a public registry. Add it to `[registries] allow` in vigilante.toml if it needs no credentials.",
                    origin(url)
                );
                return self.missing(&lockfile_path, (request.project, key), reason);
            }
            PackageSource::Git(url) => match github_archive(url) {
                Some(archive) => {
                    commit = Some(archive.commit);
                    archive.download
                }
                None => {
                    let reason =
                        format!("{label} comes from a git repository outside GitHub (`{url}`).");
                    return self.missing(&lockfile_path, (request.project, key), reason);
                }
            },
            PackageSource::Bundled => {
                let reason = format!("{label} is bundled inside another package.");
                return self.missing(&lockfile_path, (request.project, key), reason);
            }
            PackageSource::Unsupported(source) => {
                let reason = format!("{label} comes from `{source}`, which Vigilante cannot read.");
                return self.missing(&lockfile_path, (request.project, key), reason);
            }
        };
        let is_git = commit.is_some();
        let fetched = self.registry.fetch(FetchRequest {
            package: PackageId {
                ecosystem: ECOSYSTEM,
                name: &package.name,
                version: commit.as_deref().unwrap_or(&package.version),
            },
            download: &download,
            keep: &|path| is_code_file(path) || path.ends_with(MANIFEST),
        });
        let folder = match fetched {
            Ok(Fetched::Folder(folder)) => package_root(&folder, MANIFEST),
            Ok(Fetched::NotFound) => {
                let reason = format!("{label} is not at `{}`.", download.url);
                return self.missing(&lockfile_path, (request.project, key), reason);
            }
            Err(error) if is_git => {
                let reason = format!("{label} could not be downloaded from GitHub: {error}");
                return self.missing(&lockfile_path, (request.project, key), reason);
            }
            Err(error) => return Err(error),
        };
        let manifest =
            Manifest::parse(&std::fs::read_to_string(folder.join(MANIFEST)).unwrap_or_default());
        self.installed.push(Installed {
            name: package.name.clone(),
            files: folder_files(&folder)?,
            folder,
            manifest,
            project: request.project,
            key: key.clone(),
        });
        let index = self.installed.len() - 1;
        self.installed_at
            .insert((request.project, key), Some(index));
        Ok(Located::Package(index))
    }

    fn missing(
        &mut self,
        source: &str,
        place: (usize, String),
        reason: String,
    ) -> Result<Located, Error> {
        self.skip(source, reason);
        self.installed_at.insert(place, None);
        Ok(Located::Missing)
    }
}

fn origin(url: &str) -> String {
    let after_scheme = url.split_once("://").map_or(url, |(_, rest)| rest);
    let host = after_scheme.split('/').next().unwrap_or(after_scheme);
    let scheme = url.split_once("://").map_or("https", |(scheme, _)| scheme);
    format!("{scheme}://{}", host.rsplit('@').next().unwrap_or(host))
}
