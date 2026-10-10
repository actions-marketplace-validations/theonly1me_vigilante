use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::process::Command;
use std::thread;

use base64::Engine;
use serde_json::{Value, json};
use sha2::{Digest, Sha512};
use tempfile::TempDir;

use crate::support::{Run, TestRepository};

pub struct FakePackage<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub files: &'a [(&'a str, &'a str)],
}

pub struct LockEntry<'a> {
    pub key: &'a str,
    pub package: &'a FakePackage<'a>,
}

pub struct PnpmEntry<'a> {
    pub package: &'a FakePackage<'a>,
    pub dependencies: &'a [(&'a str, &'a str)],
    pub is_direct: bool,
}

pub struct NpmRegistry {
    pub base_url: String,
    integrities: BTreeMap<String, String>,
}

impl NpmRegistry {
    pub fn serve(packages: &[&FakePackage]) -> Self {
        let mut tarballs = BTreeMap::new();
        let mut integrities = BTreeMap::new();
        for package in packages {
            let bytes = tarball(package);
            let digest = base64::engine::general_purpose::STANDARD.encode(Sha512::digest(&bytes));
            integrities.insert(path_of(package), format!("sha512-{digest}"));
            tarballs.insert(path_of(package), bytes);
        }
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
        let base_url = format!("http://{}", listener.local_addr().expect("address"));
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut reader = BufReader::new(stream.try_clone().expect("clone"));
                let mut request_line = String::new();
                reader.read_line(&mut request_line).expect("request line");
                let mut header = String::new();
                while reader.read_line(&mut header).is_ok_and(|read| read > 2) {
                    header.clear();
                }
                let path = request_line.split_whitespace().nth(1).unwrap_or("/");
                let (status, body) = match tarballs.get(path) {
                    Some(body) => ("200 OK", body.clone()),
                    None => ("404 Not Found", Vec::new()),
                };
                let head = format!(
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(&body);
            }
        });
        Self {
            base_url,
            integrities,
        }
    }

    pub fn allow_config(&self) -> String {
        format!("[registries]\nallow = [\"{}\"]\n", self.base_url)
    }

    pub fn lockfile(&self, entries: &[LockEntry]) -> String {
        let mut packages = serde_json::Map::new();
        packages.insert("".to_string(), json!({ "name": "app", "version": "1.0.0" }));
        for entry in entries {
            let path = path_of(entry.package);
            packages.insert(
                entry.key.to_string(),
                json!({
                    "version": entry.package.version,
                    "resolved": format!("{}{path}", self.base_url),
                    "integrity": self.integrities.get(&path).cloned().unwrap_or_default(),
                }),
            );
        }
        let lockfile =
            json!({ "name": "app", "lockfileVersion": 3, "requires": true, "packages": packages });
        serde_json::to_string_pretty(&lockfile).expect("lockfile")
    }
}

impl NpmRegistry {
    pub fn pnpm_lockfile(&self, entries: &[PnpmEntry]) -> String {
        let mut lines = vec![
            "lockfileVersion: '9.0'".to_string(),
            String::new(),
            "importers:".to_string(),
            "  .:".to_string(),
            "    dependencies:".to_string(),
        ];
        for entry in entries.iter().filter(|entry| entry.is_direct) {
            lines.push(format!("      {}:", entry.package.name));
            lines.push(format!("        specifier: ^{}", entry.package.version));
            lines.push(format!("        version: {}", entry.package.version));
        }
        lines.push("packages:".to_string());
        for entry in entries {
            let path = path_of(entry.package);
            let integrity = self.integrities.get(&path).cloned().unwrap_or_default();
            lines.push(format!(
                "  {}@{}:",
                entry.package.name, entry.package.version
            ));
            lines.push(format!(
                "    resolution: {{integrity: {integrity}, tarball: '{}{path}'}}",
                self.base_url
            ));
        }
        lines.push("snapshots:".to_string());
        for entry in entries {
            lines.push(format!(
                "  {}@{}:",
                entry.package.name, entry.package.version
            ));
            if !entry.dependencies.is_empty() {
                lines.push("    dependencies:".to_string());
                lines.extend(
                    entry
                        .dependencies
                        .iter()
                        .map(|(name, version)| format!("      {name}: {version}")),
                );
            }
        }
        lines.join("\n") + "\n"
    }
}

pub fn run_with_cache(repository: &TestRepository, arguments: &[&str]) -> Run {
    let cache = TempDir::new().expect("cache");
    let output = Command::new(env!("CARGO_BIN_EXE_vigilante"))
        .arg("--repository")
        .arg(repository.directory.path())
        .args(arguments)
        .env("XDG_CACHE_HOME", cache.path())
        .output()
        .expect("vigilante runs");
    Run {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

pub fn report_of(run: &Run) -> Value {
    serde_json::from_str(&run.stdout)
        .unwrap_or_else(|_| panic!("JSON report: {}\n{}", run.stdout, run.stderr))
}

fn path_of(package: &FakePackage) -> String {
    let base_name = package.name.rsplit('/').next().unwrap_or(package.name);
    format!("/{}/-/{base_name}-{}.tgz", package.name, package.version)
}

fn tarball(package: &FakePackage) -> Vec<u8> {
    let manifest = json!({ "name": package.name, "version": package.version }).to_string();
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    let mut builder = tar::Builder::new(encoder);
    let manifest_entry = ("package.json", manifest.as_str());
    let has_manifest = package
        .files
        .iter()
        .any(|(path, _)| *path == "package.json");
    let generated = (!has_manifest).then_some(&manifest_entry);
    for (path, contents) in package.files.iter().chain(generated) {
        let mut header = tar::Header::new_gnu();
        header.set_size(contents.len() as u64);
        header.set_mode(0o644);
        header.set_entry_type(tar::EntryType::Regular);
        builder
            .append_data(&mut header, format!("package/{path}"), contents.as_bytes())
            .expect("append");
    }
    builder.into_inner().expect("tar").finish().expect("gzip")
}
