use super::{Linker, specifier_module, unit_of};
use crate::deep::graph::Target;
use crate::javascript::syntax::UseTarget;

impl Linker<'_> {
    pub(super) fn targets(&mut self, file: usize, target: &UseTarget) -> Vec<Target> {
        match target {
            UseTarget::Name(name) => {
                let local = self.local(file, name);
                if local.is_empty() {
                    self.binding(file, name, None)
                } else {
                    local
                }
            }
            UseTarget::Member {
                object,
                property,
                call,
            } => {
                if object == "this" {
                    return self.method(file, property);
                }
                if self.files[file].index.bindings.contains_key(object) {
                    return self.binding(file, object, Some(property));
                }
                let local = self.local(file, &format!("{object}.{property}"));
                if local.is_empty() && *call {
                    self.method(file, property)
                } else {
                    local
                }
            }
            UseTarget::Method(name) => self.method(file, name),
            UseTarget::Load(specifier) => {
                self.privileged(file, specifier, specifier_module(specifier))
            }
        }
    }

    pub(super) fn local(&self, file: usize, name: &str) -> Vec<Target> {
        let constructor = format!("{name}.constructor");
        self.files[file]
            .index
            .definitions
            .iter()
            .enumerate()
            .filter(|(_, definition)| definition.name == name || definition.name == constructor)
            .map(|(position, _)| Target::Function(self.functions[file][position]))
            .collect()
    }

    fn method(&self, file: usize, name: &str) -> Vec<Target> {
        let loaded = &self.files[file];
        let mut units = vec![unit_of(loaded)];
        for key in loaded.resolved.values() {
            if let Some(target) = self.ids.get(key) {
                let unit = unit_of(&self.files[*target]);
                if !units.contains(&unit) {
                    units.push(unit);
                }
            }
        }
        units
            .iter()
            .filter_map(|unit| self.by_name.get(unit)?.get(name))
            .flatten()
            .map(|id| Target::Function(*id))
            .collect()
    }
}
