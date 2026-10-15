use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use crate::file_server::{allow_config, serve_built};
use crate::support::{Run, TestRepository};

pub const DIFF: &[&str] = &[
    "diff", "--base", "main", "--head", "feature", "--format", "json",
];
pub const EMPTY_APP: &str = "class App\n  def main\n  end\nend\n";

pub struct FakeGem<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub dependencies: &'a [&'a str],
    pub files: &'a [(&'a str, &'a str)],
}

pub const SPAWNER: FakeGem = FakeGem {
    name: "spawner",
    version: "3.0.0",
    dependencies: &[],
    files: &[(
        "lib/spawner.rb",
        "module Spawner\n  def self.spawn_it\n    system(\"ls\")\n  end\nend\n",
    )],
};

pub const RUNNER: FakeGem = FakeGem {
    name: "runner",
    version: "1.2.0",
    dependencies: &["spawner"],
    files: &[(
        "lib/runner.rb",
        "require \"spawner\"\n\nmodule Runner\n  def self.start\n    Spawner.spawn_it\n  end\nend\n",
    )],
};

pub struct GemServer {
    pub base_url: String,
    checksums: BTreeMap<String, String>,
}

impl GemServer {
    pub fn serve(gems: &[&FakeGem]) -> Self {
        let mut checksums = BTreeMap::new();
        let base_url = serve_built(|_| {
            let mut files = BTreeMap::new();
            for gem in gems {
                let bytes = gem_bytes(gem);
                let digest: String = Sha256::digest(&bytes)
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect();
                let api = format!(
                    "/api/v2/rubygems/{}/versions/{}.json",
                    gem.name, gem.version
                );
                files.insert(
                    api.clone(),
                    format!(
                        "{{\"platform\": \"java\", \"sha\": \"{}\"}}",
                        "0".repeat(64)
                    )
                    .into_bytes(),
                );
                files.insert(
                    format!("{api}?platform=ruby"),
                    format!("{{\"platform\": \"ruby\", \"sha\": \"{digest}\"}}").into_bytes(),
                );
                files.insert(
                    format!("/downloads/{}-{}.gem", gem.name, gem.version),
                    bytes,
                );
                checksums.insert(gem.name.to_string(), digest);
            }
            files
        });
        Self {
            base_url,
            checksums,
        }
    }

    pub fn lockfile(&self, gems: &[&FakeGem]) -> String {
        let mut text = format!("GEM\n  remote: {}/\n  specs:\n", self.base_url);
        for gem in gems {
            text.push_str(&format!("    {} ({})\n", gem.name, gem.version));
            for dependency in gem.dependencies {
                text.push_str(&format!("      {dependency}\n"));
            }
        }
        text.push_str("\nPLATFORMS\n  ruby\n\nDEPENDENCIES\n");
        for gem in gems {
            text.push_str(&format!("  {}\n", gem.name));
        }
        text
    }

    pub fn checksums(&self, gems: &[&FakeGem]) -> String {
        let lines: String = gems
            .iter()
            .map(|gem| {
                format!(
                    "  {} ({}) sha256={}\n",
                    gem.name,
                    gem.version,
                    self.checksums.get(gem.name).cloned().unwrap_or_default()
                )
            })
            .collect();
        format!("\nCHECKSUMS\n{lines}")
    }

    pub fn allow_config(&self) -> String {
        allow_config(&self.base_url)
    }
}

pub struct Change<'a> {
    pub base: &'a [(&'a str, &'a str)],
    pub head: &'a [(&'a str, &'a str)],
}

pub fn diff(change: Change) -> Run {
    let repository = TestRepository::new();
    repository.write(&[("app.rb", EMPTY_APP)]);
    repository.write(change.base);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(change.head);
    repository.commit("change");
    repository.run_with_cache(DIFF)
}

fn gem_bytes(gem: &FakeGem) -> Vec<u8> {
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    let mut data = tar::Builder::new(encoder);
    for (path, contents) in gem.files {
        let mut header = tar::Header::new_gnu();
        header.set_size(contents.len() as u64);
        header.set_mode(0o644);
        header.set_entry_type(tar::EntryType::Regular);
        data.append_data(&mut header, path, contents.as_bytes())
            .expect("append");
    }
    let data = data.into_inner().expect("tar").finish().expect("gzip");
    let mut outer = tar::Builder::new(Vec::new());
    let mut header = tar::Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_mode(0o644);
    header.set_entry_type(tar::EntryType::Regular);
    outer
        .append_data(&mut header, "data.tar.gz", data.as_slice())
        .expect("gem data");
    outer.into_inner().expect("gem")
}
