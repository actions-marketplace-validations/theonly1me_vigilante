use super::{FileUse, Linker};
use crate::deep::graph::{Sink, Target};
use crate::deep::program::UseTarget;
use crate::deep::program::typed::{CallSite, Typed, TypedTarget};
use crate::{Capability, Power};

const UNANALYZED_SINK: &str = "a call that types cannot follow";
const UNANALYZED_RULE: &str = "typed-unanalyzed";

pub(super) struct TypedUse<'a> {
    pub item: FileUse<'a>,
    pub line: usize,
}

impl Linker<'_> {
    pub(super) fn call_targets(&mut self, wanted: TypedUse) -> Vec<Target> {
        let TypedUse { item, line } = wanted;
        let typed = self
            .types
            .zip(callee_of(item.target))
            .and_then(|(types, callee)| {
                types.call(CallSite {
                    file: &self.files[item.file].key,
                    line,
                    callee,
                })
            });
        match typed {
            Some(Typed::Targets(targets)) => {
                let found: Vec<Target> = targets
                    .iter()
                    .flat_map(|target| self.definitions(target))
                    .collect();
                if found.is_empty() {
                    self.targets(item)
                } else {
                    found
                }
            }
            Some(Typed::Unanalyzed) => vec![Target::Sink(self.graph.add_sink(Sink {
                name: UNANALYZED_SINK.to_string(),
                power: Power::new(Capability::Unanalyzed),
                rule: UNANALYZED_RULE.to_string(),
            }))],
            None => self.targets(item),
        }
    }

    fn definitions(&self, target: &TypedTarget) -> Vec<Target> {
        let Some(file) = self.ids.get(&target.file) else {
            return Vec::new();
        };
        let loaded = &self.files[*file];
        (1..loaded.index.definitions.len())
            .filter(|position| {
                let definition = &loaded.index.definitions[*position];
                definition.simple == target.name
                    && loaded.line_of(definition.range.start) == target.line
            })
            .map(|position| Target::Function(self.functions[*file][position]))
            .collect()
    }
}

fn callee_of(target: &UseTarget) -> Option<&str> {
    match target {
        UseTarget::Name(name) | UseTarget::Method(name) => Some(name),
        UseTarget::Member { property, .. } => Some(property),
        UseTarget::Load(_) => None,
    }
}
