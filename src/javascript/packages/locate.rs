use super::decide::{Decision, Planned};
use super::{Installed, Locate, Located, Packages};
use crate::Error;
use crate::javascript::files::{folder_files, package_root};
use crate::javascript::manifest::Manifest;
use crate::registry::Fetched;

const MANIFEST: &str = "package.json";

impl Packages<'_> {
    pub fn locate(&mut self, request: Locate) -> Result<Located, Error> {
        let lockfile_path = self.projects.project(request.project).file.clone();
        match self.decide(&request) {
            Decision::Located(located) => Ok(located),
            Decision::NotInLockfile => {
                if request.is_from_repository && request.is_declared {
                    let reason = format!("`{}` is not in the lockfile.", request.name);
                    self.skip(&lockfile_path, reason);
                }
                Ok(Located::Missing)
            }
            Decision::Skip { key, reason } => {
                self.missing(&lockfile_path, (request.project, key), reason)
            }
            Decision::Fetch(planned) => self.install(request.project, planned, &lockfile_path),
        }
    }

    fn install(
        &mut self,
        project: usize,
        planned: Planned,
        lockfile_path: &str,
    ) -> Result<Located, Error> {
        let label = planned.label();
        let folder = match super::decide::fetch(self.registry, &planned) {
            Ok(Fetched::Folder(folder)) => package_root(&folder, MANIFEST),
            Ok(Fetched::NotFound) => {
                let reason = format!("{label} is not at `{}`.", planned.download.url);
                return self.missing(lockfile_path, (project, planned.key), reason);
            }
            Err(error) if planned.commit.is_some() => {
                let reason = format!("{label} could not be downloaded from GitHub: {error}");
                return self.missing(lockfile_path, (project, planned.key), reason);
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
