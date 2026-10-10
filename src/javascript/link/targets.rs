use super::bindings::{BindingUse, PrivilegedUse};
use super::{FileUse, InFile, Linker, specifier_module, unit_of};
use crate::deep::graph::Target;
use crate::javascript::syntax::UseTarget;

impl Linker<'_> {
    pub(super) fn targets(&mut self, item: FileUse) -> Vec<Target> {
        let file = item.file;
        match item.target {
            UseTarget::Name(name) => {
                let local = self.local(InFile { file, name });
                if local.is_empty() {
                    self.binding(BindingUse {
                        file,
                        local: name,
                        member: None,
                    })
                } else {
                    local
                }
            }
            UseTarget::Member {
                object,
                property,
                call,
            } => {
                let method = InFile {
                    file,
                    name: property,
                };
                if object == "this" {
                    return self.method(method);
                }
                if self.files[file].index.bindings.contains_key(object) {
                    return self.binding(BindingUse {
                        file,
                        local: object,
                        member: Some(property),
                    });
                }
                let local = self.local(InFile {
                    file,
                    name: &format!("{object}.{property}"),
                });
                if local.is_empty() && *call {
                    self.method(method)
                } else {
                    local
                }
            }
            UseTarget::Method(name) => self.method(InFile { file, name }),
            UseTarget::Load(specifier) => self.privileged(PrivilegedUse {
                file,
                specifier,
                sink_name: specifier_module(specifier),
            }),
        }
    }

    pub(super) fn local(&self, wanted: InFile) -> Vec<Target> {
        let InFile { file, name } = wanted;
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

    fn method(&self, wanted: InFile) -> Vec<Target> {
        let InFile { file, name } = wanted;
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
