use super::environment::{GemLocation, GemQuery, InstalledFile};
use super::resolver::RubyResolver;
use crate::Error;
use crate::deep::program::{FileKey, Root};
use crate::repository_path::Folder;

const LIBRARY: &str = "lib/";
const TEST_FOLDERS: &[&str] = &["test/", "spec/"];

pub enum Searched {
    Found(FileKey),
    Unreadable,
    Missing,
}

pub struct Lookup<'a> {
    pub from: &'a FileKey,
    pub relative: String,
    pub hints: Vec<String>,
    pub scan_all: bool,
}

impl RubyResolver<'_> {
    pub fn find(&mut self, lookup: Lookup) -> Result<Searched, Error> {
        let Lookup {
            from,
            relative,
            mut hints,
            scan_all,
        } = lookup;
        let library = format!("{LIBRARY}{relative}");
        if let Root::Package(package) = from.root
            && self.environment.installed(package).files.contains(&library)
        {
            return Ok(Searched::Found(FileKey {
                root: from.root.clone(),
                path: library,
            }));
        }
        let name = relative.rsplit('/').next().unwrap_or(&relative).to_string();
        let mut own: Vec<&String> = self
            .own_by_name
            .get(&name)
            .into_iter()
            .flatten()
            .filter(|path| *path == &relative || path.ends_with(&format!("/{relative}")))
            .collect();
        own.sort_by_key(|path| {
            (
                TEST_FOLDERS.iter().any(|folder| path.contains(folder)),
                path.len(),
            )
        });
        if let Some(path) = own.first() {
            return Ok(Searched::Found(FileKey {
                root: Root::Repository,
                path: (*path).clone(),
            }));
        }
        let lock = match from.root {
            Root::Repository => self.environment.lock_for(&from.path)?,
            Root::Package(package) => Some(self.environment.installed(package).lock),
        };
        let Some(lock) = lock else {
            return Ok(Searched::Missing);
        };
        let mut searched = Searched::Missing;
        if scan_all {
            self.environment.prefetch_lock(lock);
            for name in self.environment.locked_names(lock) {
                if !hints.contains(&name) {
                    hints.push(name);
                }
            }
        }
        for hint in hints {
            let found = match self.environment.gem(GemQuery { lock, name: &hint })? {
                Some(GemLocation::Package(package)) => Some(FileKey {
                    root: Root::Package(package),
                    path: library.clone(),
                }),
                Some(GemLocation::Own(folder)) => {
                    searched = Searched::Unreadable;
                    folder.joined(&library).map(|path| FileKey {
                        root: Root::Repository,
                        path,
                    })
                }
                Some(GemLocation::Unreadable) => {
                    searched = Searched::Unreadable;
                    None
                }
                None => None,
            };
            if let Some(found) = found.filter(|file| self.environment.exists(file)) {
                return Ok(Searched::Found(found));
            }
        }
        if let Some(package) = self.environment.installed_with(InstalledFile {
            lock,
            path: &library,
        }) {
            return Ok(Searched::Found(FileKey {
                root: Root::Package(package),
                path: library,
            }));
        }
        Ok(searched)
    }
}
