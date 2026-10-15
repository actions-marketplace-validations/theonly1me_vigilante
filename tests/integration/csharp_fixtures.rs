use std::collections::BTreeMap;
use std::io::Write;

use base64::Engine;
use sha2::{Digest, Sha512};

use crate::file_server::{allow_config, serve_built};
use crate::support::{Run, TestRepository};

pub const DIFF: &[&str] = &[
    "diff", "--base", "main", "--head", "feature", "--format", "json",
];
pub const TOOLS_ASSEMBLY: &[u8] = include_bytes!("../fixtures/csharp/Acme.Tools.dll");
pub const EMPTY_PROGRAM: &str =
    "namespace App;\n\npublic class Program\n{\n    public static void Main()\n    {\n    }\n}\n";

pub struct FakeNuget<'a> {
    pub id: &'a str,
    pub version: &'a str,
    pub assembly: Option<&'a [u8]>,
    pub dependencies: &'a [(&'a str, &'a str)],
}

pub const TOOLS: FakeNuget = FakeNuget {
    id: "Acme.Tools",
    version: "1.0.0",
    assembly: Some(TOOLS_ASSEMBLY),
    dependencies: &[("Acme.Extra", "2.0.0")],
};

pub const META: FakeNuget = FakeNuget {
    id: "Acme.Meta",
    version: "1.0.0",
    assembly: None,
    dependencies: &[("Acme.Tools", "1.0.0")],
};

pub const EXTRA_WITH_OLD_TOOLS: FakeNuget = FakeNuget {
    id: "Acme.Extra",
    version: "2.0.0",
    assembly: None,
    dependencies: &[("Acme.Tools", "0.5.0")],
};

pub const EXTRA: FakeNuget = FakeNuget {
    id: "Acme.Extra",
    version: "2.0.0",
    assembly: None,
    dependencies: &[],
};

pub struct Feed {
    pub base_url: String,
    hashes: BTreeMap<String, String>,
}

impl Feed {
    pub fn serve(packages: &[&FakeNuget]) -> Self {
        let mut hashes = BTreeMap::new();
        let base_url = serve_built(|base_url| {
            let mut files = BTreeMap::new();
            files.insert(
                "/v3/index.json".to_string(),
                format!("{{\"resources\": [{{\"@id\": \"{base_url}/flat/\", \"@type\": \"PackageBaseAddress/3.0.0\"}}, {{\"@id\": \"{base_url}/registration/\", \"@type\": \"RegistrationsBaseUrl/3.6.0\"}}]}}").into_bytes(),
            );
            for package in packages {
                let id = package.id.to_lowercase();
                let version = package.version;
                let nuspec = nuspec(package);
                let bytes = nupkg(Nupkg {
                    package,
                    nuspec: &nuspec,
                });
                let hash = base64::engine::general_purpose::STANDARD.encode(Sha512::digest(&bytes));
                files.insert(format!("/flat/{id}/{version}/{id}.{version}.nupkg"), bytes);
                files.insert(
                    format!("/flat/{id}/{version}/{id}.nuspec"),
                    nuspec.into_bytes(),
                );
                files.insert(
                    format!("/registration/{id}/{version}.json"),
                    format!("{{\"catalogEntry\": \"{base_url}/catalog/{id}.{version}.json\"}}")
                        .into_bytes(),
                );
                files.insert(
                    format!("/catalog/{id}.{version}.json"),
                    format!(
                        "{{\"packageHash\": \"{hash}\", \"packageHashAlgorithm\": \"SHA512\"}}"
                    )
                    .into_bytes(),
                );
                hashes.insert(package.id.to_string(), hash);
            }
            files
        });
        Self { base_url, hashes }
    }

    pub fn lockfile(&self, packages: &[&FakeNuget]) -> String {
        let entries: Vec<String> = packages
            .iter()
            .map(|package| {
                format!(
                    "\"{}\": {{ \"type\": \"Transitive\", \"resolved\": \"{}\", \"contentHash\": \"{}\" }}",
                    package.id,
                    package.version,
                    self.hashes.get(package.id).cloned().unwrap_or_default()
                )
            })
            .collect();
        format!(
            "{{ \"version\": 1, \"dependencies\": {{ \"net8.0\": {{ {} }} }} }}",
            entries.join(", ")
        )
    }

    pub fn configuration(&self) -> String {
        format!(
            "<configuration><packageSources><clear /><add key=\"fake\" value=\"{}/v3/index.json\" /></packageSources></configuration>",
            self.base_url
        )
    }

    pub fn allow_config(&self) -> String {
        allow_config(&self.base_url)
    }
}

pub fn project(references: &[&FakeNuget]) -> String {
    let items: String = references
        .iter()
        .map(|package| {
            format!(
                "<PackageReference Include=\"{}\" Version=\"{}\" />",
                package.id, package.version
            )
        })
        .collect();
    format!(
        "<Project Sdk=\"Microsoft.NET.Sdk\"><PropertyGroup><TargetFramework>net8.0</TargetFramework></PropertyGroup><ItemGroup>{items}</ItemGroup></Project>"
    )
}

pub struct Change<'a> {
    pub base: &'a [(&'a str, &'a str)],
    pub head: &'a [(&'a str, &'a str)],
}

pub fn diff(change: Change) -> Run {
    let repository = TestRepository::new();
    repository.write(&[("Program.cs", EMPTY_PROGRAM)]);
    repository.write(change.base);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(change.head);
    repository.commit("change");
    repository.run_with_cache(DIFF)
}

fn nuspec(package: &FakeNuget) -> String {
    let dependencies: String = package
        .dependencies
        .iter()
        .map(|(id, version)| format!("<dependency id=\"{id}\" version=\"{version}\" />"))
        .collect();
    format!(
        "<?xml version=\"1.0\"?><package><metadata><id>{}</id><version>{}</version><dependencies><group targetFramework=\".NETStandard2.0\">{dependencies}</group></dependencies></metadata></package>",
        package.id, package.version
    )
}

struct Nupkg<'a> {
    package: &'a FakeNuget<'a>,
    nuspec: &'a str,
}

fn nupkg(input: Nupkg) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default();
    writer
        .start_file(format!("{}.nuspec", input.package.id), options)
        .expect("zip entry");
    writer
        .write_all(input.nuspec.as_bytes())
        .expect("zip write");
    if let Some(assembly) = input.package.assembly {
        writer
            .start_file(
                format!("lib/netstandard2.0/{}.dll", input.package.id),
                options,
            )
            .expect("zip entry");
        writer.write_all(assembly).expect("zip write");
    }
    writer.finish().expect("zip").into_inner()
}
