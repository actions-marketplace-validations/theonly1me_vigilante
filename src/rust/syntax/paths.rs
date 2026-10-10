use std::collections::HashSet;

use super::uses::{SEPARATOR, segments};
use crate::deep::program::{Binding, FileIndex, Imported, Load, UseTarget};

pub const PATH_KEY: &str = "path:";
const SELF_TYPE: &str = "Self";
const KEYWORDS: &[&str] = &["crate", "self", "super"];
const PRELUDE_NAMES: &[&str] = &[
    "Box",
    "Vec",
    "String",
    "Option",
    "Result",
    "Some",
    "None",
    "Ok",
    "Err",
    "Default",
    "Clone",
    "From",
    "Into",
    "TryFrom",
    "TryInto",
    "Iterator",
    "IntoIterator",
    "FromIterator",
    "ToString",
    "ToOwned",
    "AsRef",
    "AsMut",
    "PartialEq",
    "PartialOrd",
    "Ord",
    "Extend",
];
const PRIMITIVE_TYPES: &[&str] = &[
    "bool", "char", "str", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64",
    "i128", "isize", "f32", "f64",
];

pub struct PathUse<'a> {
    pub index: &'a mut FileIndex,
    pub text: &'a str,
    pub impl_type: Option<&'a str>,
    pub inline_modules: &'a HashSet<String>,
    pub line: usize,
}

pub struct MemberCall {
    pub object: String,
    pub property: String,
}

pub fn member(call: MemberCall) -> UseTarget {
    UseTarget::Member {
        object: call.object,
        property: call.property,
        call: true,
    }
}

pub fn path_target(path: PathUse) -> Option<UseTarget> {
    let PathUse {
        index,
        text,
        impl_type,
        inline_modules,
        line,
    } = path;
    let parts = segments(text);
    let last = parts.last()?.clone();
    if parts.len() == 1 {
        return Some(UseTarget::Name(last));
    }
    if PRIMITIVE_TYPES.contains(&parts[0].as_str()) || PRELUDE_NAMES.contains(&parts[0].as_str()) {
        return None;
    }
    if parts[0] == SELF_TYPE {
        return impl_type.map(|owner| {
            member(MemberCall {
                object: owner.to_string(),
                property: parts[1].clone(),
            })
        });
    }
    if inline_modules.contains(&parts[0]) {
        return Some(UseTarget::Name(last));
    }
    let type_position = parts
        .iter()
        .position(|part| part.starts_with(char::is_uppercase));
    let (module_length, imported, property) = match type_position {
        Some(0) => {
            return Some(member(MemberCall {
                object: parts[0].clone(),
                property: parts[1].clone(),
            }));
        }
        Some(position) => (
            position,
            Imported::Named(parts[position].clone()),
            parts.get(position + 1)?.clone(),
        ),
        None => (parts.len() - 1, Imported::Module, last),
    };
    let key_length = match imported {
        Imported::Named(_) => module_length + 1,
        Imported::Module => module_length,
    };
    let specifier = module_specifier(ModulePath {
        index,
        parts: &parts[..module_length],
    });
    let key = format!("{PATH_KEY}{}", parts[..key_length].join(SEPARATOR));
    index.loads.push(Load {
        specifier: specifier.clone(),
        line,
    });
    index.bindings.insert(
        key.clone(),
        Binding {
            specifier,
            imported,
        },
    );
    Some(member(MemberCall {
        object: key,
        property,
    }))
}

struct ModulePath<'a> {
    index: &'a FileIndex,
    parts: &'a [String],
}

fn module_specifier(path: ModulePath) -> String {
    let ModulePath { index, parts } = path;
    let first = &parts[0];
    let base = match index.bindings.get(first) {
        Some(binding) if !KEYWORDS.contains(&first.as_str()) => match &binding.imported {
            Imported::Named(name) => format!("{}{SEPARATOR}{name}", binding.specifier),
            Imported::Module => binding.specifier.clone(),
        },
        _ => first.clone(),
    };
    std::iter::once(base)
        .chain(parts[1..].iter().cloned())
        .collect::<Vec<_>>()
        .join(SEPARATOR)
}
