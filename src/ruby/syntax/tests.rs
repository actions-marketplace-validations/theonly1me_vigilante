use ast_grep_core::tree_sitter::LanguageExt;
use ast_grep_language::SupportLang;

use super::index;
use crate::deep::program::{Binding, Definition, Export, FileIndex, Imported, UseTarget};

const SOURCE: &str = r#"require "net/http"
require_relative "tools/shell"

module Acme
  module Tools
    class Runner < Base
      def self.build(name)
        new(name).start
      end

      def initialize(name)
        @client = Faraday.new
      end

      def start
        helper
        self.check
        Shell.run("ls")
        Acme::Tools::Shell.run
        @client.get("x")
        client = Faraday.new
        client.post("y")
        items.each { |item| process(item) }
      end

      class << self
        def configure; end
      end

      private

      def helper; end
    end
  end
end
"#;

fn indexed(source: &str) -> FileIndex {
    let document = SupportLang::Ruby.ast_grep(source);
    index(&document.root())
}

fn targets_of(definition: Option<&Definition>) -> Vec<String> {
    definition
        .map(|definition| {
            definition
                .uses
                .iter()
                .map(|item| match &item.target {
                    UseTarget::Name(name) => format!("name {name}"),
                    UseTarget::Member {
                        object, property, ..
                    } => format!("member {object} {property}"),
                    UseTarget::Method(name) => format!("method {name}"),
                    UseTarget::Load(name) => format!("load {name}"),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn definition<'a>(index: &'a FileIndex) -> impl Fn(&str) -> Option<&'a Definition> {
    move |name| {
        index
            .definitions
            .iter()
            .find(|definition| definition.name == name)
    }
}

#[test]
fn methods_and_requires_become_definitions_loads_and_exports() {
    let index = indexed(SOURCE);

    let names: Vec<&str> = index
        .definitions
        .iter()
        .map(|definition| definition.name.as_str())
        .collect();
    assert_eq!(
        names,
        vec![
            "init",
            "Runner.build",
            "Runner.new",
            "Runner.start",
            "Runner.configure",
            "Runner.helper"
        ]
    );
    let loads: Vec<&str> = index
        .loads
        .iter()
        .map(|load| load.specifier.as_str())
        .collect();
    assert!(
        loads.starts_with(&["net/http", "./tools/shell"]),
        "{loads:?}"
    );
    assert_eq!(
        targets_of(definition(&index)("init")),
        vec!["load net/http", "load ./tools/shell"]
    );
    for name in ["Acme", "Tools", "Runner"] {
        assert!(index.exports.contains(&Export::Local {
            exported: name.to_string(),
            local: name.to_string(),
        }));
    }
}

#[test]
fn calls_resolve_through_self_constants_and_variable_types() {
    let index = indexed(SOURCE);

    assert_eq!(
        targets_of(definition(&index)("Runner.start")),
        vec![
            "member Runner helper",
            "member Runner check",
            "member Shell run",
            "member Acme::Tools::Shell run",
            "member Faraday get",
            "member Faraday new",
            "member Faraday post",
            "method each",
            "member Runner process",
        ]
    );
    assert_eq!(
        targets_of(definition(&index)("Runner.build")),
        vec!["method start", "member Runner new"]
    );
}

#[test]
fn constants_bind_to_the_module_nesting() {
    let index = indexed(SOURCE);

    let binding = |specifier: &str, name: &str| Binding {
        specifier: specifier.to_string(),
        imported: Imported::Named(name.to_string()),
    };
    assert_eq!(
        index.bindings.get("Shell"),
        Some(&binding(
            "Shell@Acme::Tools::Runner,Acme::Tools,Acme,",
            "Shell"
        ))
    );
    assert_eq!(
        index.bindings.get("Acme::Tools::Shell"),
        Some(&binding("Shell@Acme::Tools", "Shell"))
    );
    assert_eq!(index.bindings.get("Runner"), None);
}
