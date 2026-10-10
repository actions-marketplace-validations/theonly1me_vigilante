use super::{Linker, Unit, specifier_module, unit_of};
use crate::Power;
use crate::deep::graph::{Edge, Function, FunctionId, Owner, Sink, Target};
use crate::javascript::syntax::{Export, Imported};

const MAXIMUM_FORWARDS: usize = 8;
const DEFAULT_EXPORT: &str = "default";

struct ExportedLocal {
    file: usize,
    local: String,
}

impl Linker<'_> {
    pub(super) fn binding(
        &mut self,
        file: usize,
        local: &str,
        member: Option<&str>,
    ) -> Vec<Target> {
        let Some(binding) = self.files[file].index.bindings.get(local).cloned() else {
            return Vec::new();
        };
        let module = specifier_module(&binding.specifier);
        let sink_name = match (&binding.imported, member) {
            (Imported::Module, None) => module,
            (Imported::Module, Some(member)) => format!("{module}.{member}"),
            (Imported::Named(name), _) => format!("{module}.{name}"),
        };
        let privileged = self.privileged(file, &binding.specifier, sink_name);
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
        let exported = self.exported(target, &wanted, 0);
        let mut found: Vec<Target> = exported
            .iter()
            .flat_map(|item| match inner {
                Some(member) => self.local(item.file, &format!("{}.{member}", item.local)),
                None => self.local(item.file, &item.local),
            })
            .collect();
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

    pub(super) fn privileged(&mut self, file: usize, specifier: &str, name: String) -> Vec<Target> {
        let Some(privileges) = self.files[file].privileged.get(specifier) else {
            return Vec::new();
        };
        privileges
            .iter()
            .map(|privilege| Sink {
                name: name.clone(),
                power: Power::new(privilege.capability),
                rule: privilege.rule.clone(),
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|sink| Target::Sink(self.graph.add_sink(sink)))
            .collect()
    }

    fn exported(&self, file: usize, name: &str, depth: usize) -> Vec<ExportedLocal> {
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
                        found.extend(self.exported(target, imported, depth + 1));
                    }
                }
                Export::Forward {
                    exported: None,
                    specifier,
                    ..
                } if name != DEFAULT_EXPORT => {
                    if let Some(target) = forwarded(specifier) {
                        found.extend(self.exported(target, name, depth + 1));
                    }
                }
                _ => {}
            }
        }
        found
    }

    fn fallback(&mut self, package: usize) -> FunctionId {
        if let Some(id) = self.fallbacks.get(&package) {
            return *id;
        }
        let name = self.package_names[package].clone();
        let id = self.graph.add_function(Function {
            name: format!("{name}.*"),
            owner: Owner::Dependency {
                package: name.clone(),
            },
            path: name,
        });
        let members = self
            .in_unit
            .get(&Unit::Package(package))
            .cloned()
            .unwrap_or_default();
        for member in members {
            self.graph.add_edge(
                id,
                Edge {
                    line: 1,
                    target: Target::Function(member),
                },
            );
        }
        self.fallbacks.insert(package, id);
        id
    }
}
