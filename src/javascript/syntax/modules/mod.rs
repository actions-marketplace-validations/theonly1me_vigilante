mod exports;
mod imports;
mod nodes;

use std::collections::HashMap;

use super::{Binding, Export, FileIndex, Imported, Load, UseTarget, line_of};
use crate::code::SyntaxNode;

#[derive(Default)]
pub struct ModuleItems {
    bindings: HashMap<String, Binding>,
    exports: Vec<Export>,
    loads: Vec<Load>,
}

#[derive(Default)]
pub struct Outcome {
    pub load: Option<UseTarget>,
    pub skipped_child: Option<usize>,
    pub is_leaf: bool,
}

impl ModuleItems {
    pub fn read(&mut self, node: &SyntaxNode) -> Outcome {
        match node.kind().as_ref() {
            "import_statement" => self.import(node),
            "export_statement" => self.export(node),
            "call_expression" => self.call(node),
            "assignment_expression" => {
                self.assignment(node);
                Outcome::default()
            }
            "variable_declarator" => {
                self.declarator(node);
                Outcome::default()
            }
            _ => Outcome::default(),
        }
    }

    pub fn finish(self, index: &mut FileIndex) {
        index.bindings = self.bindings;
        index.exports = self.exports;
        index.loads = self.loads;
    }

    pub(super) fn bind(&mut self, local: &str, specifier: &str, imported: Imported) {
        self.bindings.insert(
            local.to_string(),
            Binding {
                specifier: specifier.to_string(),
                imported,
            },
        );
    }

    pub(super) fn local(&mut self, exported: &str, local: &str) {
        self.exports.push(Export::Local {
            exported: exported.to_string(),
            local: local.to_string(),
        });
    }

    pub(super) fn load(&mut self, specifier: &str, node: &SyntaxNode) {
        self.loads.push(Load {
            specifier: specifier.to_string(),
            line: line_of(node),
        });
    }

    pub(super) fn forward(&mut self, specifier: &str, names: (Option<String>, Option<String>)) {
        self.exports.push(Export::Forward {
            exported: names.0,
            specifier: specifier.to_string(),
            imported: names.1,
        });
    }
}
