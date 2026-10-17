use super::references::{is_called, is_storage_target};
use crate::code::SyntaxNode;
use crate::deep::program::{FileIndex, Use, UseTarget};

const RECEIVERS: &[&str] = &["self", "cls"];
const THIS: &str = "this";

pub struct Chain {
    pub definition: usize,
    pub line: usize,
    pub start: usize,
    pub parts: Vec<String>,
    pub call: bool,
    pub skipped_child: Option<usize>,
}

pub fn chain_of(attribute: &SyntaxNode) -> Option<Chain> {
    if is_storage_target(attribute) {
        return None;
    }
    let mut parts = Vec::new();
    let mut current = attribute.clone();
    loop {
        match current.kind().as_ref() {
            "attribute" => {
                parts.push(current.field("attribute")?.text().to_string());
                current = current.field("object")?;
            }
            "identifier" => {
                parts.push(current.text().to_string());
                break;
            }
            _ => return None,
        }
    }
    parts.reverse();
    Some(Chain {
        definition: 0,
        line: 0,
        start: 0,
        parts,
        call: is_called(attribute),
        skipped_child: attribute.field("object").map(|object| object.node_id()),
    })
}

pub struct ChainUses<'a> {
    pub index: &'a mut FileIndex,
    pub chains: Vec<Chain>,
}

struct ChainIn<'a> {
    index: &'a FileIndex,
    chain: &'a Chain,
}

pub fn resolve_chains(uses: ChainUses) {
    let ChainUses { index, chains } = uses;
    for chain in chains {
        let targets = targets_of(ChainIn {
            index,
            chain: &chain,
        });
        let uses = &mut index.definitions[chain.definition].uses;
        uses.extend(targets.into_iter().map(|target| Use {
            line: chain.line,
            start: chain.start,
            target,
        }));
    }
}

fn targets_of(input: ChainIn) -> Vec<UseTarget> {
    let ChainIn { index, chain } = input;
    let parts = &chain.parts;
    let last = parts.len() - 1;
    let method = || chain.call.then(|| UseTarget::Method(parts[last].clone()));
    if RECEIVERS.contains(&parts[0].as_str()) {
        let member = UseTarget::Member {
            object: THIS.to_string(),
            property: parts[1].clone(),
            call: chain.call && last == 1,
        };
        return [Some(member), (last > 1).then(method).flatten()]
            .into_iter()
            .flatten()
            .collect();
    }
    let bound = (1..parts.len())
        .rev()
        .find(|length| index.bindings.contains_key(&parts[..*length].join(".")));
    match bound {
        Some(length) => {
            let member = UseTarget::Member {
                object: parts[..length].join("."),
                property: parts[length].clone(),
                call: chain.call && length == last,
            };
            [Some(member), (length < last).then(method).flatten()]
                .into_iter()
                .flatten()
                .collect()
        }
        None if last == 1 => vec![UseTarget::Member {
            object: parts[0].clone(),
            property: parts[1].clone(),
            call: chain.call,
        }],
        None => method().into_iter().collect(),
    }
}
