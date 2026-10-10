use super::decide::{Decision, Planned};
use super::{Installed, Locate, Located, Packages};
use crate::Error;
use crate::deep::Skip;
use crate::javascript::files::{folder_files, package_root};
use crate::javascript::manifest::Manifest;
use crate::registry::Fetched;

const MANIFEST: &str = "package.json";

struct Install<'a> {
    project: usize,
    planned: Planned,
    lockfile_path: &'a str,
}

struct Missing<'a> {
    lockfile_path: &'a str,
    project: usize,
    key: String,
    reason: String,
}

impl Packages<'_> {
    pub fn locate(&mut self, request: Locate) -> Result<Located, Error> {
        let lockfile_path = self.projects.project(request.project).file.clone();
        match self.decide(&request) {
            Decision::Located(located) => Ok(located),
            Decision::NotInLockfile => {
                if request.is_from_repository && request.declared.is_some() {
                    self.skip(Skip {
                        source: lockfile_path,
                        reason: format!("`{}` is not in the lockfile.", request.name),
                    });
                }
                Ok(Located::Missing)
            }
            Decision::Skip { key, reason } => self.missing(Missing {
                lockfile_path: &lockfile_path,
                project: request.project,
                key,
                reason,
            }),
            Decision::Fetch(planned) => self.install(Install {
                project: request.project,
                planned,
                lockfile_path: &lockfile_path,
            }),
        }
    }

    fn install(&mut self, install: Install) -> Result<Located, Error> {
        let Install {
            project,
            planned,
            lockfile_path,
        } = install;
        let label = planned.label();
        let missing = |reason: String| Missing {
            lockfile_path,
            project,
            key: planned.key.clone(),
            reason,
        };
        let folder = match planned.fetch_from(self.registry) {
            Ok(Fetched::Folder(folder)) => package_root(&folder),
            Ok(Fetched::NotFound) => {
                let reason = format!("{label} is not at `{}`.", planned.download.url);
                return self.missing(missing(reason));
            }
            Err(error) if planned.commit.is_some() => {
                let reason = format!("{label} could not be downloaded from GitHub: {error}");
                return self.missing(missing(reason));
            }
            Err(error) => return Err(error),
        };
        let manifest =
            Manifest::parse(&std::fs::read_to_string(folder.join(MANIFEST)).unwrap_or_default());
        self.installed.push(Installed {
            name: planned.package.name.clone(),
            files: folder_files(&folder)?,
            folder,
            manifest,
            project,
            key: planned.key.clone(),
        });
        let index = self.installed.len() - 1;
        self.installed_at
            .insert((project, planned.key), Some(index));
        Ok(Located::Package(index))
    }

    fn missing(&mut self, missing: Missing) -> Result<Located, Error> {
        self.skip(Skip {
            source: missing.lockfile_path.to_string(),
            reason: missing.reason,
        });
        self.installed_at
            .insert((missing.project, missing.key), None);
        Ok(Located::Missing)
    }
}
