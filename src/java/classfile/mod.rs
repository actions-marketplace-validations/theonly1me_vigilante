mod bytes;
mod code;
mod constants;
mod skeleton;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod writer;

pub use skeleton::skeleton;

use bytes::Reader;
use constants::{Constant, ConstantPool, MethodReference};

const MAGIC: u32 = 0xCAFE_BABE;
const OBJECT: &str = "java/lang/Object";
const NATIVE: u16 = 0x0100;
const CODE: &str = "Code";
const BOOTSTRAP_METHODS: &str = "BootstrapMethods";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CallKind {
    Static,
    Special,
    Virtual,
    Interface,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Call {
    pub class: String,
    pub name: String,
    pub kind: CallKind,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Method {
    pub name: String,
    pub is_native: bool,
    pub calls: Vec<Call>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ClassFile {
    pub name: String,
    pub supertypes: Vec<String>,
    pub methods: Vec<Method>,
}

pub fn read(data: &[u8]) -> Option<ClassFile> {
    let mut reader = Reader::new(data);
    if reader.u32()? != MAGIC {
        return None;
    }
    reader.skip(4)?;
    let pool = ConstantPool::read(&mut reader)?;
    reader.skip(2)?;
    let name = pool.class_name(reader.u16()?)?.to_string();
    let mut supertypes: Vec<String> = pool
        .class_name(reader.u16()?)
        .filter(|superclass| *superclass != OBJECT)
        .map(str::to_string)
        .into_iter()
        .collect();
    for _ in 0..reader.u16()? {
        supertypes.push(pool.class_name(reader.u16()?)?.to_string());
    }
    let fields = reader.u16()?;
    for _ in 0..fields {
        reader.skip(6)?;
        skip_attributes(&mut reader)?;
    }
    let method_count = reader.u16()?;
    let mut methods = Vec::new();
    let mut raw_codes = Vec::new();
    for _ in 0..method_count {
        let flags = reader.u16()?;
        let method_name = pool.utf8(reader.u16()?)?.to_string();
        reader.skip(2)?;
        let mut code = None;
        for _ in 0..reader.u16()? {
            let attribute = pool.utf8(reader.u16()?)?;
            let length = reader.u32()? as usize;
            let body = reader.bytes(length)?;
            if attribute == CODE {
                code = Some(body);
            }
        }
        raw_codes.push(code);
        methods.push(Method {
            name: method_name,
            is_native: flags & NATIVE != 0,
            calls: Vec::new(),
        });
    }
    let mut bootstrap = Vec::new();
    for _ in 0..reader.u16().unwrap_or(0) {
        let attribute = pool.utf8(reader.u16()?)?;
        let length = reader.u32()? as usize;
        let body = reader.bytes(length)?;
        if attribute == BOOTSTRAP_METHODS {
            bootstrap =
                bootstrap_targets(BootstrapAttribute { pool: &pool, body }).unwrap_or_default();
        }
    }
    for (method, code) in methods.iter_mut().zip(raw_codes) {
        if let Some(code) = code {
            method.calls = code::calls(code::CodeRequest {
                pool: &pool,
                attribute: code,
                bootstrap: &bootstrap,
            })
            .unwrap_or_default();
        }
    }
    Some(ClassFile {
        name,
        supertypes,
        methods,
    })
}

fn skip_attributes(reader: &mut Reader) -> Option<()> {
    for _ in 0..reader.u16()? {
        reader.skip(2)?;
        let length = reader.u32()? as usize;
        reader.skip(length)?;
    }
    Some(())
}

struct BootstrapAttribute<'a> {
    pool: &'a ConstantPool,
    body: &'a [u8],
}

fn bootstrap_targets(attribute: BootstrapAttribute) -> Option<Vec<Vec<Call>>> {
    let BootstrapAttribute { pool, body } = attribute;
    let mut reader = Reader::new(body);
    let mut targets = Vec::new();
    for _ in 0..reader.u16()? {
        reader.skip(2)?;
        let mut calls = Vec::new();
        for _ in 0..reader.u16()? {
            if let Some(Constant::MethodHandle { reference }) = pool.get(reader.u16()?)
                && let Some(call) = pool.method_ref(MethodReference {
                    index: *reference,
                    kind: CallKind::Static,
                })
            {
                calls.push(call);
            }
        }
        targets.push(calls);
    }
    Some(targets)
}
