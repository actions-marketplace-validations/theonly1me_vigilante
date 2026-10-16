use super::privileged::PrivilegedUse;
use super::{InFile, Linker, Unit, specifier_module, unit_of};
use crate::deep::graph::Target;
use crate::deep::program::{Export, Imported};

const MAXIMUM_FORWARDS: usize = 8;
const DEFAULT_EXPORT: &str = "default";

struct ExportedLocal {
    file: usize,
    local: String,
}

pub struct BindingUse<'a> {
    pub file: usize,
    pub local: &'a str,
    pub member: Option<&'a str>,
}

struct ExportQuery<'a> {
    file: usize,
    name: &'a str,
    depth: usize,
}

impl Linker<'_> {
    pub(super) fn binding(&mut self, wanted: BindingUse) -> Vec<Target> {
        let BindingUse {
            file,
            local,
            member,
        } = wanted;
        let Some(binding) = self.files[file].index.bindings.get(local).cloned() else {
            return Vec::new();
        };
        let module = specifier_module(&binding.specifier);
        let sink_name = match (&binding.imported, member) {
            (Imported::Module, None) => module,
            (Imported::Module, Some(member)) => format!("{module}.{member}"),
            (Imported::Named(name), _) => format!("{module}.{name}"),
        };
        let used_member = match (&binding.imported, member) {
            (Imported::Named(name), _) => Some(name.as_str()),
            (Imported::Module, member) => member,
        };
        let privileged = self.privileged(PrivilegedUse {
            file,
            specifier: &binding.specifier,
            member: used_member,
            sink_name,
        });
        if !privileged.is_empty() {
            return privileged;
        }
        let Some(target) = self.files[file]
            .resolved
            .get(&binding.specifier)
            .and_then(|key| self.ids.get(key))
            .copied()
        else {
            return Vec::new();
        };
        let (wanted, inner) = match (&binding.imported, member) {
            (Imported::Module, None) => (DEFAULT_EXPORT.to_string(), None),
            (Imported::Module, Some(member)) => (member.to_string(), None),
            (Imported::Named(name), member) => (name.clone(), member),
        };
        let exported = self.exported(ExportQuery {
            file: target,
            name: &wanted,
            depth: 0,
        });
        let mut found: Vec<Target> = exported
            .iter()
            .flat_map(|item| match inner {
                Some(member) => self.local(InFile {
                    file: item.file,
                    name: &format!("{}.{member}", item.local),
                }),
                None => self.local(InFile {
                    file: item.file,
                    name: &item.local,
                }),
            })
            .collect();
        if found.is_empty() && self.language.finds_members_by_type() {
            let unit = unit_of(&self.files[target]);
            return inner
                .and_then(|member| {
                    self.by_full_name
                        .get(&unit)?
                        .get(&format!("{wanted}.{member}"))
                })
                .map(|ids| ids.iter().map(|id| Target::Function(*id)).collect())
                .unwrap_or_default();
        }
        if found.is_empty() {
            let unit = unit_of(&self.files[target]);
            let simple = inner.unwrap_or(&wanted);
            found = self
                .by_name
                .get(&unit)
                .and_then(|names| names.get(simple))
                .map(|ids| ids.iter().map(|id| Target::Function(*id)).collect())
                .unwrap_or_default();
            if found.is_empty()
                && let Unit::Package(package) = unit
            {
                found.push(Target::Function(self.fallback(package)));
            }
        }
        found
    }

    fn exported(&self, query: ExportQuery) -> Vec<ExportedLocal> {
        let ExportQuery { file, name, depth } = query;
        if depth > MAXIMUM_FORWARDS {
            return Vec::new();
        }
        let loaded = &self.files[file];
        let forwarded = |specifier: &str| {
            loaded
                .resolved
                .get(specifier)
                .and_then(|key| self.ids.get(key))
                .copied()
        };
        let mut found = Vec::new();
        for export in &loaded.index.exports {
            match export {
                Export::Local { exported, local } if exported == name => {
                    found.push(ExportedLocal {
                        file,
                        local: local.clone(),
                    })
                }
                Export::Forward {
                    exported: Some(exported),
                    specifier,
                    imported: Some(imported),
                } if exported == name => {
                    if let Some(target) = forwarded(specifier) {
                        found.extend(self.exported(ExportQuery {
                            file: target,
                            name: imported,
                            depth: depth + 1,
                        }));
                    }
                }
                Export::Forward {
                    exported: None,
                    specifier,
                    ..
                } if name != DEFAULT_EXPORT => {
                    if let Some(target) = forwarded(specifier) {
                        found.extend(self.exported(ExportQuery {
                            file: target,
                            name,
                            depth: depth + 1,
                        }));
                    }
                }
                _ => {}
            }
        }
        found
    }
}
