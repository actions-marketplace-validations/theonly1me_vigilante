use std::collections::BTreeSet;

use super::declarations::Declarations;
use crate::deep::program::{Binding, FileIndex, Imported, Load, UseTarget};

pub const CANDIDATE_MARK: char = '@';
const CANDIDATE_SEPARATOR: &str = ",";

pub struct BindingInput<'a> {
    pub index: &'a mut FileIndex,
    pub declarations: &'a Declarations,
    pub namespaces: &'a [String],
}

pub fn bind_types(input: BindingInput) {
    let BindingInput {
        index,
        declarations,
        namespaces,
    } = input;
    let objects: BTreeSet<String> = index
        .definitions
        .iter()
        .flat_map(|definition| &definition.uses)
        .filter_map(|item| match &item.target {
            UseTarget::Member { object, .. } => Some(object.clone()),
            _ => None,
        })
        .chain(
            declarations
                .supertypes
                .iter()
                .flat_map(|supertypes| supertypes.names.iter().cloned()),
        )
        .collect();
    let chain: Vec<String> = namespaces
        .first()
        .map(|namespace| {
            let parts: Vec<&str> = namespace.split('.').collect();
            (1..=parts.len())
                .rev()
                .map(|length| parts[..length].join("."))
                .collect()
        })
        .unwrap_or_default();
    for object in objects {
        if declarations.local_types.contains(&object) {
            continue;
        }
        let qualified = declarations
            .aliases
            .get(&object)
            .cloned()
            .unwrap_or_else(|| object.clone());
        let (prefix, name) = match qualified.rsplit_once('.') {
            Some((prefix, name)) => (Some(prefix.to_string()), name.to_string()),
            None => (None, qualified.clone()),
        };
        let candidates: Vec<String> = match (&prefix, declarations.aliases.contains_key(&object)) {
            (Some(prefix), true) => vec![prefix.clone()],
            (Some(prefix), false)
                if object.split('.').next().is_some_and(|first| {
                    chain
                        .iter()
                        .any(|namespace| namespace.split('.').next() == Some(first))
                }) =>
            {
                vec![prefix.clone()]
            }
            (Some(prefix), false) => chain
                .iter()
                .map(|namespace| format!("{namespace}.{prefix}"))
                .chain([prefix.clone()])
                .collect(),
            (None, _) => chain
                .iter()
                .cloned()
                .chain(declarations.usings.iter().cloned())
                .chain([String::new()])
                .collect(),
        };
        let specifier = format!(
            "{name}{CANDIDATE_MARK}{}",
            candidates.join(CANDIDATE_SEPARATOR)
        );
        index.loads.push(Load {
            specifier: specifier.clone(),
            line: 1,
        });
        index.bindings.insert(
            object,
            Binding {
                specifier,
                imported: Imported::Named(name),
            },
        );
    }
}
