use ast_grep_core::tree_sitter::LanguageExt;
use ast_grep_language::SupportLang;

use super::index;
use crate::deep::program::{Binding, Definition, Export, FileIndex, Imported, UseTarget};

fn indexed(source: &str) -> FileIndex {
    let document = SupportLang::Rust.ast_grep(source);
    index(&document.root())
}

fn named(path: &str) -> Binding {
    let (specifier, name) = path.rsplit_once("::").expect("path");
    Binding {
        specifier: specifier.to_string(),
        imported: Imported::Named(name.to_string()),
    }
}

fn module(specifier: &str) -> Binding {
    Binding {
        specifier: specifier.to_string(),
        imported: Imported::Module,
    }
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
fn use_trees_become_bindings_loads_and_exports() {
    let index = indexed(
        "use std::process::{self, Command as Cmd};\nuse crate::net::{fetch, client::Client};\npub use self::inner::Thing;\nuse reqwest;\nextern crate foo as bar;\npub use super::shared::*;\nmod child;\n",
    );

    assert_eq!(index.bindings.get("process"), Some(&named("std::process")));
    assert_eq!(
        index.bindings.get("Cmd"),
        Some(&named("std::process::Command"))
    );
    assert_eq!(
        index.bindings.get("fetch"),
        Some(&named("crate::net::fetch"))
    );
    assert_eq!(
        index.bindings.get("Client"),
        Some(&named("crate::net::client::Client"))
    );
    assert_eq!(index.bindings.get("reqwest"), Some(&module("reqwest")));
    assert_eq!(index.bindings.get("bar"), Some(&module("foo")));
    assert!(index.exports.contains(&Export::Forward {
        exported: Some("Thing".to_string()),
        specifier: "self::inner".to_string(),
        imported: Some("Thing".to_string()),
    }));
    assert!(index.exports.contains(&Export::Forward {
        exported: None,
        specifier: "super::shared".to_string(),
        imported: None,
    }));
    let loads: Vec<&str> = index
        .loads
        .iter()
        .map(|load| load.specifier.as_str())
        .collect();
    for expected in [
        "crate::net",
        "crate::net::fetch",
        "crate::net::client::Client",
        "self::child",
        "super::shared",
    ] {
        assert!(loads.contains(&expected), "{expected} in {loads:?}");
    }
}

#[test]
fn functions_methods_and_calls_become_definitions_and_uses() {
    let index = indexed(
        "use crate::net;\n\npub fn helper() {}\n\nimpl<T> Runner<T> for Worker {\n    fn go(&self) {\n        self.step();\n        Self::make();\n        helper();\n        net::fetch();\n        crate::web::Client::new();\n        items.iter().count();\n        Cache::<u8>::new();\n    }\n}\n\ntrait Tool {\n    fn run(&self) {}\n    fn stop(&self);\n}\n",
    );

    let names: Vec<&str> = index
        .definitions
        .iter()
        .map(|definition| definition.name.as_str())
        .collect();
    assert_eq!(names, vec!["init", "helper", "Worker.go", "Tool.run"]);
    assert_eq!(
        targets_of(definition(&index)("Worker.go")),
        vec![
            "member Worker step",
            "member Worker make",
            "name helper",
            "member path:net fetch",
            "member path:crate::web::Client new",
            "method count",
            "method iter",
            "member Cache new",
        ]
    );
    assert_eq!(index.bindings.get("path:net"), Some(&module("crate::net")));
    assert_eq!(
        index.bindings.get("path:crate::web::Client"),
        Some(&named("crate::web::Client"))
    );
    assert!(index.exports.contains(&Export::Local {
        exported: "helper".to_string(),
        local: "helper".to_string(),
    }));
}

#[test]
fn test_items_are_left_out_and_their_ranges_are_ignored() {
    let source = "fn real() {}\n\n#[cfg(test)]\nmod tests {\n    fn check() { unsafe { run() } }\n}\n\n#[test]\nfn alone() {}\n";
    let index = indexed(source);

    let names: Vec<&str> = index
        .definitions
        .iter()
        .map(|definition| definition.name.as_str())
        .collect();
    assert_eq!(names, vec!["init", "real"]);
    let unsafe_start = source.find("unsafe").expect("unsafe");
    assert!(
        index
            .ignored
            .iter()
            .any(|range| range.contains(&unsafe_start))
    );
    assert!(
        !index
            .ignored
            .iter()
            .any(|range| range.contains(&source.find("real").expect("real")))
    );
}

#[test]
fn primitive_types_and_single_name_uses_are_not_crates() {
    let index = indexed(
        "pub(crate) use generate_array;\n\nfn widen(value: u32) -> u64 {\n    u64::from(value).max(i64::MAX as u64)\n}\n",
    );

    let loads: Vec<&str> = index
        .loads
        .iter()
        .map(|load| load.specifier.as_str())
        .collect();
    assert_eq!(loads, Vec::<&str>::new());
    assert!(
        index.bindings.keys().all(|key| !key.starts_with("path:")),
        "{:?}",
        index.bindings
    );
}

#[test]
fn calls_on_prelude_types_and_traits_have_no_targets() {
    let index = indexed(
        "fn build() -> Vec<u8> {\n    let settings: Settings = Default::default();\n    let mut bytes = Vec::new();\n    bytes.push(String::from(\"a\").len() as u8);\n    bytes\n}\n",
    );

    assert_eq!(
        targets_of(definition(&index)("build")),
        vec!["method push", "method len"]
    );
}
