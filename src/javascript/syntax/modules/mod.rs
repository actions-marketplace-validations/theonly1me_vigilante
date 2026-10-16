mod common_js;
mod exports;
mod imports;
mod nodes;

pub use nodes::constant_strings;
use nodes::{arguments, is_loader, loaded_specifier};

use std::collections::HashMap;

use super::{Binding, Export, FileIndex, Imported, Load, UseTarget, line_of};
use crate::code::SyntaxNode;

#[derive(Default)]
pub struct ModuleItems {
    bindings: HashMap<String, Binding>,
    exports: Vec<Export>,
    loads: Vec<Load>,
    unresolved_loads: Vec<usize>,
    constants: HashMap<String, String>,
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

    pub fn with_constants(constants: HashMap<String, String>) -> Self {
        Self {
            constants,
            ..Self::default()
        }
    }

    pub(super) fn loaded_specifier(&self, node: &SyntaxNode) -> Option<String> {
        loaded_specifier(node).or_else(|| {
            if !is_loader(node) {
                return None;
            }
            let [argument] = arguments(node).try_into().ok()?;
            (argument.kind() == "identifier")
                .then(|| self.constants.get(argument.text().as_ref()).cloned())
                .flatten()
        })
    }

    pub fn finish(self, index: &mut FileIndex) {
        index.bindings = self.bindings;
        index.exports = self.exports;
        index.loads = self.loads;
        index.unresolved_loads = self.unresolved_loads;
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

    pub(super) fn unresolved(&mut self, node: &SyntaxNode) {
        self.unresolved_loads.push(line_of(node));
    }

    pub(super) fn load(&mut self, site: LoadSite) {
        self.loads.push(Load {
            specifier: site.specifier.to_string(),
            line: line_of(site.node),
        });
    }
}
