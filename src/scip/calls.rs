use std::collections::HashMap;

use super::Index;
use crate::deep::program::{FileKey, Typed, TypedCall, TypedCalls, TypedTarget};

const CALL_SUFFIX: &str = "().";
const LOCAL: &str = "local ";
const DESCRIPTOR_ENDS: &[char] = &['#', '/', '.', ']'];

pub struct CallsInput<'a> {
    pub index: &'a Index,
    pub keys: &'a HashMap<String, FileKey>,
    pub external: &'a dyn Fn(&str) -> Option<TypedTarget>,
}

pub fn typed_calls(input: CallsInput) -> TypedCalls {
    let CallsInput {
        index,
        keys,
        external,
    } = input;
    let mut definitions: HashMap<&str, TypedTarget> = HashMap::new();
    for document in &index.documents {
        let Some(file) = keys.get(&document.path) else {
            continue;
        };
        for occurrence in document
            .occurrences
            .iter()
            .filter(|item| item.is_definition)
        {
            if let Some(name) = called_name(&occurrence.symbol) {
                definitions.insert(
                    &occurrence.symbol,
                    TypedTarget {
                        file: file.clone(),
                        name: name.to_string(),
                        line: occurrence.line + 1,
                    },
                );
            }
        }
    }
    let mut calls = TypedCalls::default();
    for document in &index.documents {
        let Some(file) = keys.get(&document.path) else {
            continue;
        };
        for occurrence in document
            .occurrences
            .iter()
            .filter(|item| !item.is_definition)
        {
            let target = definitions
                .get(occurrence.symbol.as_str())
                .cloned()
                .or_else(|| external(&occurrence.symbol));
            let Some(target) = target else {
                continue;
            };
            calls.add(TypedCall {
                file: file.clone(),
                line: occurrence.line + 1,
                callee: target.name.clone(),
                typed: Typed::Targets(vec![target]),
            });
        }
    }
    calls
}

fn called_name(symbol: &str) -> Option<&str> {
    if symbol.starts_with(LOCAL) {
        return None;
    }
    let path = symbol.strip_suffix(CALL_SUFFIX)?;
    let name = path.rsplit(DESCRIPTOR_ENDS).next()?.trim_matches('`');
    (!name.is_empty()).then_some(name)
}

pub fn without_call_suffix(descriptor: &str) -> Option<&str> {
    descriptor.strip_suffix(CALL_SUFFIX)
}
