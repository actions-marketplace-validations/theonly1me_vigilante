use std::collections::BTreeMap;

use crate::deep::program::{Binding, FileIndex, Imported, Load};

pub const CANDIDATE_MARK: char = '@';
pub const NAMESPACE_SEPARATOR: &str = "::";
const CANDIDATE_SEPARATOR: &str = ",";

pub struct ConstantInput<'a> {
    pub index: &'a mut FileIndex,
    pub constants: BTreeMap<String, Vec<String>>,
    pub local: &'a [String],
}

pub fn bind_constants(input: ConstantInput) {
    let ConstantInput {
        index,
        constants,
        local,
    } = input;
    for (constant, nesting) in constants {
        if local.contains(&constant) {
            continue;
        }
        let trimmed = constant.trim_start_matches(NAMESPACE_SEPARATOR);
        let (name, candidates) = match trimmed.rsplit_once(NAMESPACE_SEPARATOR) {
            Some((prefix, name)) => (name.to_string(), vec![prefix.to_string()]),
            None => {
                let chain: Vec<String> = (1..=nesting.len())
                    .rev()
                    .map(|length| nesting[..length].join(NAMESPACE_SEPARATOR))
                    .chain([String::new()])
                    .collect();
                (trimmed.to_string(), chain)
            }
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
            constant,
            Binding {
                specifier,
                imported: Imported::Named(name),
            },
        );
    }
}
