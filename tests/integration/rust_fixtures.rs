use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use crate::file_server::{allow_config, serve_built};
use crate::support::{Run, TestRepository};

pub const DIFF: &[&str] = &[
    "diff", "--base", "main", "--head", "feature", "--format", "json",
];
pub const EMPTY_MAIN: &str = "fn main() {}\n";

pub struct FakeCrate<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub dependencies: &'a [&'a str],
    pub files: &'a [(&'a str, &'a str)],
}

pub const SPAWNER: FakeCrate = FakeCrate {
    name: "spawner",
    version: "2.0.0",
    dependencies: &[],
    files: &[(
        "src/lib.rs",
        "use std::process::Command;\n\npub fn spawn_it() {\n    Command::new(\"ls\").status();\n}\n",
    )],
};

pub const RUNNER: FakeCrate = FakeCrate {
    name: "runner",
    version: "1.0.0",
    dependencies: &["spawner"],
    files: &[
        ("src/lib.rs", "mod work;\n\npub use self::work::start;\n"),
        (
            "src/work.rs",
            "pub fn start() {\n    spawner::spawn_it();\n}\n",
        ),
    ],
};

pub struct Registry {
    pub base_url: String,
    checksums: BTreeMap<String, String>,
}

impl Registry {
    pub fn serve(crates: &[&FakeCrate]) -> Self {
        let mut checksums = BTreeMap::new();
        let base_url = serve_built(|base_url| {
            let mut files = BTreeMap::new();
            files.insert(
                "/index/config.json".to_string(),
                format!("{{\"dl\": \"{base_url}/api/v1/crates\"}}").into_bytes(),
            );
            for item in crates {
                let bytes = crate_bytes(item);
                let digest: String = Sha256::digest(&bytes)
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect();
                checksums.insert(item.name.to_string(), digest);
                files.insert(
                    format!("/api/v1/crates/{}/{}/download", item.name, item.version),
                    bytes,
                );
            }
            files
        });
        Self {
            base_url,
            checksums,
        }
    }

    pub fn lockfile(&self, crates: &[&FakeCrate]) -> String {
        let names: Vec<String> = crates
            .iter()
            .filter(|item| {
                crates
                    .iter()
                    .all(|other| !other.dependencies.contains(&item.name))
            })
            .map(|item| format!(" \"{}\",\n", item.name))
            .collect();
        let mut text = format!(
            "version = 4\n\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\ndependencies = [\n{}]\n",
            names.concat()
        );
        for item in crates {
            let dependencies: String = item
                .dependencies
                .iter()
                .map(|name| format!(" \"{name}\",\n"))
                .collect();
            text.push_str(&format!(
                "\n[[package]]\nname = \"{}\"\nversion = \"{}\"\nsource = \"sparse+{}/index/\"\nchecksum = \"{}\"\ndependencies = [\n{dependencies}]\n",
                item.name,
                item.version,
                self.base_url,
                self.checksums.get(item.name).cloned().unwrap_or_default()
            ));
        }
        text
    }

    pub fn allow_config(&self) -> String {
        allow_config(&self.base_url)
    }
}

pub fn app_manifest(dependencies: &[&str]) -> String {
    let lines: String = dependencies
        .iter()
        .map(|name| format!("{name} = \"*\"\n"))
        .collect();
    format!("[package]\nname = \"app\"\nversion = \"0.1.0\"\n\n[dependencies]\n{lines}")
}

pub struct Change<'a> {
    pub base: &'a [(&'a str, &'a str)],
    pub head: &'a [(&'a str, &'a str)],
}

pub fn diff(change: Change) -> Run {
    let repository = TestRepository::new();
    repository.write(&[("src/main.rs", EMPTY_MAIN)]);
    repository.write(change.base);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(change.head);
    repository.commit("change");
    repository.run_with_cache(DIFF)
}

fn crate_bytes(item: &FakeCrate) -> Vec<u8> {
    let dependencies: String = item
        .dependencies
        .iter()
        .map(|name| format!("{name} = \"*\"\n"))
        .collect();
    let manifest = format!(
        "[package]\nname = \"{}\"\nversion = \"{}\"\n\n[dependencies]\n{dependencies}",
        item.name, item.version
    );
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    let mut builder = tar::Builder::new(encoder);
    for (path, contents) in item
        .files
        .iter()
        .chain([&("Cargo.toml", manifest.as_str())])
    {
        let mut header = tar::Header::new_gnu();
        header.set_size(contents.len() as u64);
        header.set_mode(0o644);
        header.set_entry_type(tar::EntryType::Regular);
        builder
            .append_data(
                &mut header,
                format!("{}-{}/{path}", item.name, item.version),
                contents.as_bytes(),
            )
            .expect("append");
    }
    builder.into_inner().expect("tar").finish().expect("gzip")
}
