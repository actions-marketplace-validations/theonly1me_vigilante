use std::collections::HashMap;

use super::{Linker, unit_of};
use crate::deep::graph::Function;

impl Linker<'_> {
    pub(super) fn add_functions(&mut self) {
        for (file_position, file) in self.files.iter().enumerate() {
            let unit = unit_of(file);
            let mut ids = Vec::new();
            let mut in_file: HashMap<String, Vec<usize>> = HashMap::new();
            for (position, definition) in file.index.definitions.iter().enumerate() {
                let id = self.graph.add_function(Function {
                    name: format!("{}.{}", self.language.stem(&file.display), definition.name),
                    owner: file.owner.clone(),
                    path: file.display.clone(),
                });
                if position > 0 {
                    self.by_name
                        .entry(unit.clone())
                        .or_default()
                        .entry(definition.simple.clone())
                        .or_default()
                        .push(id);
                    self.by_full_name
                        .entry(unit.clone())
                        .or_default()
                        .entry(definition.name.clone())
                        .or_default()
                        .push(id);
                }
                in_file.entry(definition.name.clone()).or_default().push(id);
                self.places.push((file_position, position));
                self.in_unit.entry(unit.clone()).or_default().push(id);
                ids.push(id);
            }
            self.functions.push(ids);
            self.in_file.push(in_file);
        }
    }
}
