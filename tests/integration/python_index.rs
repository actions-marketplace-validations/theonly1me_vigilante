use std::collections::BTreeMap;
use std::io::Write;

use sha2::{Digest, Sha256};

use crate::file_server::{allow_config, serve_files};

pub struct FakeWheel<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub files: &'a [(&'a str, &'a str)],
}

pub struct PythonIndex {
    pub base_url: String,
    hashes: BTreeMap<String, String>,
}

impl PythonIndex {
    pub fn serve(wheels: &[&FakeWheel]) -> Self {
        let mut files = BTreeMap::new();
        let mut hashes = BTreeMap::new();
        for wheel in wheels {
            let bytes = wheel_bytes(wheel);
            let digest: String = Sha256::digest(&bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect();
            hashes.insert(file_name(wheel), digest);
            files.insert(format!("/{}", file_name(wheel)), bytes);
        }
        Self {
            base_url: serve_files(files),
            hashes,
        }
    }

    pub fn allow_config(&self) -> String {
        allow_config(&self.base_url)
    }

    pub fn uv_lock(&self, wheels: &[&FakeWheel]) -> String {
        let mut text = String::from(
            "version = 1\n\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\nsource = { virtual = \".\" }\n",
        );
        for wheel in wheels {
            let name = file_name(wheel);
            let hash = self.hashes.get(&name).cloned().unwrap_or_default();
            text.push_str(&format!(
                "\n[[package]]\nname = \"{}\"\nversion = \"{}\"\nsource = {{ registry = \"https://pypi.org/simple\" }}\nwheels = [{{ url = \"{}/{name}\", hash = \"sha256:{hash}\" }}]\n",
                wheel.name, wheel.version, self.base_url
            ));
        }
        text
    }
}

fn file_name(wheel: &FakeWheel) -> String {
    format!("{}-{}-py3-none-any.whl", wheel.name, wheel.version)
}

fn wheel_bytes(wheel: &FakeWheel) -> Vec<u8> {
    let top_level = (
        format!("{}-{}.dist-info/top_level.txt", wheel.name, wheel.version),
        format!("{}\n", wheel.name),
    );
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let entries = wheel
        .files
        .iter()
        .map(|(path, text)| (path.to_string(), text.to_string()))
        .chain([top_level]);
    for (path, contents) in entries {
        writer
            .start_file(path, zip::write::SimpleFileOptions::default())
            .expect("zip entry");
        writer.write_all(contents.as_bytes()).expect("zip write");
    }
    writer.finish().expect("zip").into_inner()
}
