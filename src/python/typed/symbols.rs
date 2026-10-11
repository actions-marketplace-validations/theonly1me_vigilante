use std::collections::HashMap;

use super::project::{PACKAGE_PREFIX, Project};
use crate::deep::program::{Program, Typed, TypedCall, TypedCalls, TypedTarget};
use crate::scip::Index;

const CALL_SUFFIX: &str = "().";
const LOCAL: &str = "local ";
const DESCRIPTOR_ENDS: &[char] = &['#', '/', '.'];
const SYMBOL_PARTS: usize = 5;

pub struct Indexed<'a, 'p> {
    pub index: &'a Index,
    pub project: &'a Project,
    pub program: &'a Program<'p>,
}

pub fn typed_calls(indexed: Indexed) -> TypedCalls {
    let Indexed {
        index,
        project,
        program,
    } = indexed;
    let mut definitions: HashMap<&str, TypedTarget> = HashMap::new();
    for document in &index.documents {
        let Some(file) = project.keys.get(&document.path) else {
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
        let Some(file) = project.keys.get(&document.path) else {
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
                .or_else(|| {
                    package_target(PackageSymbol {
                        symbol: &occurrence.symbol,
                        project,
                        program,
                    })
                });
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

struct PackageSymbol<'a, 'p> {
    symbol: &'a str,
    project: &'a Project,
    program: &'a Program<'p>,
}

fn package_target(wanted: PackageSymbol) -> Option<TypedTarget> {
    let PackageSymbol {
        symbol,
        project,
        program,
    } = wanted;
    let parts: Vec<&str> = symbol.splitn(SYMBOL_PARTS, ' ').collect();
    let [_, _, package, _, descriptors] = parts[..] else {
        return None;
    };
    let package: usize = package.strip_prefix(PACKAGE_PREFIX)?.parse().ok()?;
    let (module, member) = descriptors.split_once('/')?;
    let module = module.trim_matches('`').replace('.', "/");
    let qualified = member.strip_suffix(CALL_SUFFIX)?.replace('#', ".");
    if !qualified
        .chars()
        .all(|character| character.is_alphanumeric() || character == '_' || character == '.')
    {
        return None;
    }
    let file = [format!("{module}.py"), format!("{module}/__init__.py")]
        .into_iter()
        .find_map(|path| project.modules.get(&(package, path)))?;
    let loaded = program.files.iter().find(|loaded| &loaded.key == file)?;
    let definition = loaded
        .index
        .definitions
        .iter()
        .skip(1)
        .find(|definition| definition.name == qualified)?;
    Some(TypedTarget {
        file: file.clone(),
        name: definition.simple.clone(),
        line: loaded.line_of(definition.range.start),
    })
}

fn called_name(symbol: &str) -> Option<&str> {
    if symbol.starts_with(LOCAL) {
        return None;
    }
    let path = symbol.strip_suffix(CALL_SUFFIX)?;
    let name = path.rsplit(DESCRIPTOR_ENDS).next()?.trim_matches('`');
    (!name.is_empty()).then_some(name)
}
