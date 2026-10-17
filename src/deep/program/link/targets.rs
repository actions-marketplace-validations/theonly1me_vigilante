use super::bindings::BindingUse;
use super::privileged::PrivilegedUse;
use super::{FileUse, InFile, Linker, Unit, specifier_module, unit_of};
use crate::deep::graph::Target;
use crate::deep::program::UseTarget;

const CONSTRUCTORS: &[&str] = &["constructor", "__init__"];

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
                let full_name = format!("{object}.{property}");
                let mut local = self.local(InFile {
                    file,
                    name: &full_name,
                });
                if local.is_empty() && self.language.finds_members_by_type() {
                    local = self.typed(InFile {
                        file,
                        name: &full_name,
                    });
                }
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
                member: None,
                sink_name: specifier_module(specifier),
            }),
        }
    }

    pub(super) fn local(&self, wanted: InFile) -> Vec<Target> {
        let InFile { file, name } = wanted;
        let in_file = &self.in_file[file];
        let mut found: Vec<usize> = in_file.get(name).cloned().unwrap_or_default();
        for constructor in CONSTRUCTORS {
            if let Some(ids) = in_file.get(&format!("{name}.{constructor}")) {
                found.extend(ids);
            }
        }
        found.sort_unstable();
        found.into_iter().map(Target::Function).collect()
    }

    fn method(&self, wanted: InFile) -> Vec<Target> {
        let InFile { file, name } = wanted;
        self.units(file)
            .iter()
            .filter_map(|unit| self.by_name.get(unit)?.get(name))
            .flatten()
            .map(|id| Target::Function(*id))
            .collect()
    }

    fn typed(&self, wanted: InFile) -> Vec<Target> {
        let InFile { file, name } = wanted;
        self.units(file)
            .iter()
            .filter_map(|unit| self.by_full_name.get(unit)?.get(name))
            .flatten()
            .map(|id| Target::Function(*id))
            .collect()
    }

    fn units(&self, file: usize) -> Vec<Unit> {
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
    }
}
