use std::collections::{HashMap, HashSet};

use super::Linker;
use crate::deep::graph::Target;

pub(super) type ClassKey = (usize, String);

impl Linker<'_> {
    pub(super) fn add_hierarchy(&mut self) {
        let mut subtypes: HashMap<ClassKey, Vec<ClassKey>> = HashMap::new();
        for (file, loaded) in self.files.iter().enumerate() {
            for supertypes in &loaded.index.supertypes {
                for name in &supertypes.names {
                    let Some(binding) = loaded.index.bindings.get(name) else {
                        continue;
                    };
                    let Some(target) = loaded
                        .resolved
                        .get(&binding.specifier)
                        .and_then(|key| self.ids.get(key))
                    else {
                        continue;
                    };
                    let class = name.rsplit('.').next().unwrap_or(name).to_string();
                    subtypes
                        .entry((*target, class))
                        .or_default()
                        .push((file, supertypes.class.clone()));
                }
            }
        }
        self.subtypes = subtypes;
    }

    pub(super) fn with_overrides(&self, targets: Vec<Target>) -> Vec<Target> {
        let mut expanded = targets.clone();
        for target in targets {
            let Target::Function(id) = target else {
                continue;
            };
            let Some(&(file, position)) = self.places.get(id) else {
                continue;
            };
            let name = &self.files[file].index.definitions[position].name;
            let Some((class, method)) = name.rsplit_once('.') else {
                continue;
            };
            let mut seen: HashSet<ClassKey> = HashSet::new();
            let mut pending = vec![(file, class.to_string())];
            while let Some(current) = pending.pop() {
                for subtype in self.subtypes.get(&current).into_iter().flatten() {
                    if !seen.insert(subtype.clone()) {
                        continue;
                    }
                    let (sub_file, sub_class) = subtype;
                    for id in self.in_file[*sub_file]
                        .get(&format!("{sub_class}.{method}"))
                        .into_iter()
                        .flatten()
                    {
                        if !expanded.contains(&Target::Function(*id)) {
                            expanded.push(Target::Function(*id));
                        }
                    }
                    pending.push(subtype.clone());
                }
            }
        }
        expanded
    }
}
