mod common_js;
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

pub struct NewBinding<'a> {
    pub local: &'a str,
    pub specifier: &'a str,
    pub imported: Imported,
}

pub struct LoadSite<'a, 'r> {
    pub specifier: &'a str,
    pub node: &'a SyntaxNode<'r>,
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

    pub(super) fn bind(&mut self, binding: NewBinding) {
        self.bindings.insert(
            binding.local.to_string(),
            Binding {
                specifier: binding.specifier.to_string(),
                imported: binding.imported,
            },
        );
    }

    pub(super) fn add_export(&mut self, export: Export) {
        self.exports.push(export);
    }

    pub(super) fn load(&mut self, site: LoadSite) {
        self.loads.push(Load {
            specifier: site.specifier.to_string(),
            line: line_of(site.node),
        });
    }
}
