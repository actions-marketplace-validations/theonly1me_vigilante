use std::io::Write;

use sha1::{Digest, Sha1};

pub struct TestCall<'a> {
    pub class: &'a str,
    pub name: &'a str,
    pub is_static: bool,
}

pub struct TestMethod<'a> {
    pub name: &'a str,
    pub calls: &'a [TestCall<'a>],
}

pub struct TestClass<'a> {
    pub name: &'a str,
    pub methods: &'a [TestMethod<'a>],
}

#[derive(Default)]
struct Pool {
    bytes: Vec<u8>,
    count: u16,
}

impl Pool {
    fn add(&mut self, entry: Vec<u8>) -> u16 {
        self.count += 1;
        self.bytes.extend(entry);
        self.count
    }

    fn utf8(&mut self, text: &str) -> u16 {
        let mut entry = vec![1];
        entry.extend((text.len() as u16).to_be_bytes());
        entry.extend(text.as_bytes());
        self.add(entry)
    }

    fn class(&mut self, name: &str) -> u16 {
        let name = self.utf8(name);
        self.add([vec![7], name.to_be_bytes().to_vec()].concat())
    }

    fn method(&mut self, call: &TestCall) -> u16 {
        let class = self.class(call.class);
        let name = self.utf8(call.name);
        let descriptor = self.utf8("()V");
        let pair = self.add(
            [
                vec![12],
                name.to_be_bytes().to_vec(),
                descriptor.to_be_bytes().to_vec(),
            ]
            .concat(),
        );
        self.add(
            [
                vec![10],
                class.to_be_bytes().to_vec(),
                pair.to_be_bytes().to_vec(),
            ]
            .concat(),
        )
    }
}

pub fn class_bytes(class: &TestClass) -> Vec<u8> {
    let mut pool = Pool::default();
    let this_class = pool.class(class.name);
    let super_class = pool.class("java/lang/Object");
    let code_name = pool.utf8("Code");
    let mut methods = Vec::new();
    for method in class.methods {
        let name = pool.utf8(method.name);
        let descriptor = pool.utf8("()V");
        let mut code = Vec::new();
        for call in method.calls {
            let index = pool.method(call);
            code.push(if call.is_static { 0xb8 } else { 0xb7 });
            code.extend(index.to_be_bytes());
        }
        code.push(0xb1);
        let mut entry = vec![0, 0x09];
        entry.extend(name.to_be_bytes());
        entry.extend(descriptor.to_be_bytes());
        entry.extend(1u16.to_be_bytes());
        entry.extend(code_name.to_be_bytes());
        entry.extend(((code.len() + 12) as u32).to_be_bytes());
        entry.extend([0, 4, 0, 4]);
        entry.extend((code.len() as u32).to_be_bytes());
        entry.extend(&code);
        entry.extend([0, 0, 0, 0]);
        methods.push(entry);
    }
    let mut bytes = vec![0xCA, 0xFE, 0xBA, 0xBE, 0, 0, 0, 61];
    bytes.extend((pool.count + 1).to_be_bytes());
    bytes.extend(&pool.bytes);
    bytes.extend([0, 0x21]);
    bytes.extend(this_class.to_be_bytes());
    bytes.extend(super_class.to_be_bytes());
    bytes.extend([0, 0, 0, 0]);
    bytes.extend((methods.len() as u16).to_be_bytes());
    for method in methods {
        bytes.extend(method);
    }
    bytes.extend([0, 0]);
    bytes
}

pub fn jar_bytes(classes: &[&TestClass]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for class in classes {
        writer
            .start_file(
                format!("{}.class", class.name),
                zip::write::SimpleFileOptions::default(),
            )
            .expect("jar entry");
        writer.write_all(&class_bytes(class)).expect("jar write");
    }
    writer.finish().expect("jar").into_inner()
}

pub struct Artifact<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub pom: &'a str,
    pub classes: &'a [&'a TestClass<'a>],
}

pub fn artifact_files(artifact: Artifact) -> [(String, Vec<u8>); 3] {
    let Artifact {
        name,
        version,
        pom,
        classes,
    } = artifact;
    let folder = format!("/com/acme/{name}/{version}/{name}-{version}");
    let jar = jar_bytes(classes);
    let digest: String = Sha1::digest(&jar)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    [
        (format!("{folder}.pom"), pom.as_bytes().to_vec()),
        (format!("{folder}.jar.sha1"), digest.into_bytes()),
        (format!("{folder}.jar"), jar),
    ]
}
