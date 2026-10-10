use crate::Error;
use crate::lockfiles::python::{PythonFile, PythonPackage, PythonSource};
use crate::registry::{
    Archive, Checksum, Download, FetchRequest, Fetched, PackageId, Prefetch, Registry, Sources,
    github_archive,
};

pub const ECOSYSTEM: &str = "pypi";
pub const PYPI_JSON: &str = "https://pypi.org/pypi";
const PURE_WHEEL: &str = "-none-any.whl";
const WHEEL: &str = ".whl";
const SOURCE_ARCHIVES: &[&str] = &[".tar.gz", ".zip"];

pub enum Plan {
    Download(Planned),
    Skip(String),
    Local,
}

pub struct Planned {
    pub name: String,
    pub version: String,
    pub download: Download,
    pub is_git: bool,
}

pub struct PlanContext<'a> {
    pub registry: &'a Registry,
    pub sources: &'a Sources,
    pub index: &'a str,
    pub package: &'a PythonPackage,
}

impl Planned {
    pub fn fetch_from(&self, registry: &Registry) -> Result<Fetched, Error> {
        registry.fetch(FetchRequest {
            package: PackageId {
                ecosystem: ECOSYSTEM,
                name: &self.name,
                version: &self.version,
            },
            download: &self.download,
            keep: &is_kept,
        })
    }

    pub fn prefetch(&self) -> Prefetch {
        Prefetch {
            ecosystem: ECOSYSTEM,
            name: self.name.clone(),
            version: self.version.clone(),
            download: self.download.clone(),
            keep: is_kept,
        }
    }
}

pub fn plan(context: PlanContext) -> Result<Plan, Error> {
    let PlanContext {
        registry,
        sources,
        index,
        package,
    } = context;
    let label = format!("`{}=={}`", package.name, package.version);
    let planned = |download: Download| {
        Plan::Download(Planned {
            name: package.name.clone(),
            version: package.version.clone(),
            is_git: false,
            download,
        })
    };
    match &package.source {
        PythonSource::Files(files) => Ok(match chosen(files) {
            Some(file) if sources.allows(&file.url) => planned(download_of(file)),
            Some(file) => Plan::Skip(format!(
                "{label} comes from `{}`, which is not a public index. Add it to `[registries] allow` in vigilante.toml if it needs no credentials.",
                file.url
            )),
            None => Plan::Skip(format!("{label} has no file that Vigilante can read.")),
        }),
        PythonSource::Index { hashes } => {
            let url = format!("{index}/{}/{}/json", package.name, package.version);
            let Some(document) = registry.get_json(&url)? else {
                return Ok(Plan::Skip(format!("{label} is not on PyPI.")));
            };
            let files: Vec<PythonFile> = document["urls"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|file| {
                    Some(PythonFile {
                        url: file["url"].as_str()?.to_string(),
                        sha256: file["digests"]["sha256"].as_str().map(str::to_string),
                        size: file["size"].as_u64(),
                    })
                })
                .filter(|file| {
                    hashes.is_empty()
                        || file
                            .sha256
                            .as_ref()
                            .is_some_and(|hash| hashes.contains(hash))
                })
                .collect();
            match chosen(&files) {
                Some(file) => Ok(planned(download_of(file))),
                None if !hashes.is_empty() => Err(Error::new(format!(
                    "no file of {label} on PyPI matches the hashes in the lockfile"
                ))),
                None => Ok(Plan::Skip(format!(
                    "{label} has no file on PyPI that Vigilante can read."
                ))),
            }
        }
        PythonSource::Git(url) => Ok(match github_archive(url) {
            Some(archive) => Plan::Download(Planned {
                name: package.name.clone(),
                version: archive.commit,
                download: archive.download,
                is_git: true,
            }),
            None => Plan::Skip(format!(
                "{label} comes from a git repository outside GitHub (`{url}`)."
            )),
        }),
        PythonSource::Folder(_) => Ok(Plan::Local),
        PythonSource::Private(url) => Ok(Plan::Skip(format!(
            "{label} comes from `{url}`, which is not a public index."
        ))),
        PythonSource::Unsupported(source) => Ok(Plan::Skip(format!(
            "{label} comes from `{source}`, which Vigilante cannot read."
        ))),
    }
}

fn chosen(files: &[PythonFile]) -> Option<&PythonFile> {
    let smallest = |suffixes: &[&str]| {
        files
            .iter()
            .filter(|file| suffixes.iter().any(|suffix| file.url.ends_with(suffix)))
            .min_by_key(|file| file.size.unwrap_or(u64::MAX))
    };
    smallest(&[PURE_WHEEL])
        .or_else(|| smallest(&[WHEEL]))
        .or_else(|| smallest(SOURCE_ARCHIVES))
}

fn download_of(file: &PythonFile) -> Download {
    let is_zip = file.url.ends_with(".whl") || file.url.ends_with(".zip");
    Download {
        url: file.url.clone(),
        checksum: file.sha256.as_deref().and_then(Checksum::sha256_hex),
        archive: if is_zip {
            Archive::Zip
        } else {
            Archive::TarGzip
        },
    }
}

fn is_kept(path: &str) -> bool {
    path.ends_with(".py") || path.ends_with("/top_level.txt")
}
