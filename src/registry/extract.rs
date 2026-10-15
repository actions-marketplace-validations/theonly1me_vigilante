use std::fs;
use std::io::{Cursor, Read};
use std::path::{Component, Path, PathBuf};

use flate2::read::GzDecoder;

use super::Archive;
use crate::Error;

const MAXIMUM_FILE_BYTES: u64 = 64 * 1024 * 1024;
const MAXIMUM_TOTAL_BYTES: u64 = 1024 * 1024 * 1024;
const MAXIMUM_ENTRIES: usize = 200_000;
const GEM_DATA: &str = "data.tar.gz";

pub struct ExtractRequest<'a> {
    pub bytes: &'a [u8],
    pub archive: Archive,
    pub target: &'a Path,
    pub keep: &'a dyn Fn(&str) -> bool,
}

pub fn extract(request: ExtractRequest) -> Result<(), Error> {
    let mut writer = Writer {
        target: request.target,
        keep: request.keep,
        total_bytes: 0,
        entries: 0,
    };
    match request.archive {
        Archive::TarGzip => writer.tar(GzDecoder::new(Cursor::new(request.bytes))),
        Archive::Zip => writer.zip(request.bytes),
        Archive::Gem => {
            let data = gem_data(request.bytes)?;
            writer.tar(GzDecoder::new(Cursor::new(data)))
        }
    }
}

struct Writer<'a> {
    target: &'a Path,
    keep: &'a dyn Fn(&str) -> bool,
    total_bytes: u64,
    entries: usize,
}

impl Writer<'_> {
    fn tar(&mut self, reader: impl Read) -> Result<(), Error> {
        let mut archive = tar::Archive::new(reader);
        for entry in archive.entries().map_err(invalid)? {
            let mut entry = entry.map_err(invalid)?;
            if !entry.header().entry_type().is_file() {
                continue;
            }
            let path = entry.path().map_err(invalid)?.into_owned();
            let size = entry.size();
            self.write(Entry {
                path: &path,
                size,
                reader: &mut entry,
            })?;
        }
        Ok(())
    }

    fn zip(&mut self, bytes: &[u8]) -> Result<(), Error> {
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(invalid)?;
        for index in 0..archive.len() {
            let mut file = archive.by_index(index).map_err(invalid)?;
            if file.is_dir() || file.is_symlink() {
                continue;
            }
            let Some(path) = file.enclosed_name() else {
                let name = file.name().map(|name| name.to_string()).unwrap_or_default();
                return Err(Error::new(format!(
                    "the archive has a path outside the package: {name}"
                )));
            };
            let size = file.size();
            self.write(Entry {
                path: &path,
                size,
                reader: &mut file,
            })?;
        }
        Ok(())
    }

    fn write(&mut self, entry: Entry) -> Result<(), Error> {
        let relative = safe_relative(entry.path).ok_or_else(|| {
            Error::new(format!(
                "the archive has a path outside the package: {}",
                entry.path.display()
            ))
        })?;
        let name = relative.to_string_lossy().replace('\\', "/");
        if !(self.keep)(&name) {
            return Ok(());
        }
        self.entries += 1;
        self.total_bytes += entry.size;
        if entry.size > MAXIMUM_FILE_BYTES
            || self.total_bytes > MAXIMUM_TOTAL_BYTES
            || self.entries > MAXIMUM_ENTRIES
        {
            return Err(Error::new(format!("the archive is too large at {name}")));
        }
        let mut contents = Vec::new();
        entry
            .reader
            .take(MAXIMUM_FILE_BYTES + 1)
            .read_to_end(&mut contents)
            .map_err(invalid)?;
        let destination = self.target.join(&relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(invalid)?;
        }
        fs::write(destination, contents).map_err(invalid)
    }
}

struct Entry<'a> {
    path: &'a Path,
    size: u64,
    reader: &'a mut dyn Read,
}

fn safe_relative(path: &Path) -> Option<PathBuf> {
    let mut relative = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => relative.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    (!relative.as_os_str().is_empty()).then_some(relative)
}

fn gem_data(bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let mut archive = tar::Archive::new(Cursor::new(bytes));
    for entry in archive.entries().map_err(invalid)? {
        let mut entry = entry.map_err(invalid)?;
        if entry.path().map_err(invalid)?.as_ref() == Path::new(GEM_DATA) {
            let mut data = Vec::new();
            entry
                .by_ref()
                .take(MAXIMUM_TOTAL_BYTES)
                .read_to_end(&mut data)
                .map_err(invalid)?;
            return Ok(data);
        }
    }
    Err(Error::new(format!("the gem has no {GEM_DATA}")))
}

fn invalid(error: impl std::fmt::Display) -> Error {
    Error::new(format!("cannot read the package archive: {error}"))
}
