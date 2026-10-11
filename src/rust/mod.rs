mod cargo;
mod environment;
mod language;
mod modules;
mod resolver;
mod syntax;
mod test_modules;

use std::collections::BTreeMap;

use ast_grep_language::SupportLang;

use crate::Error;
use crate::code::{Rules, language_of};
use crate::config::Config;
use crate::deep::Analysis;
use crate::deep::program::{AnalyzeInput, Program, ProgramInput, RootsInput, selected_roots};
use crate::registry::Registry;
use crate::scan::Selection;
use crate::tree::Tree;
use environment::{EnvironmentInput, RustEnvironment};
use language::Rust;
use resolver::RustResolver;
use test_modules::{TestModulesInput, without_test_modules};

const BUILD_FILES: &[&str] = &["Cargo.toml", "Cargo.lock"];

pub struct RustRequest<'a> {
    pub tree: &'a dyn Tree,
    pub config: &'a Config,
    pub rules: &'a Rules,
    pub scope_of: &'a dyn Fn(&str) -> String,
    pub selection: &'a Selection,
}

pub fn owns(path: &str) -> bool {
    language_of(path) == Some(SupportLang::Rust)
}

pub fn analyze(request: RustRequest) -> Result<Analysis, Error> {
    let roots = selected_roots(RootsInput {
        tree: request.tree,
        config: request.config,
        scope_of: request.scope_of,
        selection: request.selection,
        is_code: owns,
        lockfiles: BUILD_FILES,
    });
    if roots.is_empty() {
        return Ok(Analysis::default());
    }
    let registry = Registry::new(Registry::default_cache_root());
    let mut resolver = RustResolver {
        environment: RustEnvironment::new(EnvironmentInput {
            tree: request.tree,
            registry: &registry,
            sources: request.config.registry_sources(),
        })?,
        scope_of: request.scope_of,
        unresolved: BTreeMap::new(),
    };
    let roots = without_test_modules(TestModulesInput {
        tree: request.tree,
        environment: &resolver.environment,
        roots,
    })?;
    let mut program = Program::new(ProgramInput {
        tree: request.tree,
        rules: request.rules,
        scope_of: request.scope_of,
        language: &Rust,
        resolver: &mut resolver,
    });
    let root_paths: Vec<String> = roots.iter().cloned().collect();
    program.load(&root_paths)?;
    let mut analysis = program.analyze(AnalyzeInput {
        roots: &roots,
        types: None,
    });
    drop(program);
    analysis.skipped = resolver.into_skipped();
    Ok(analysis)
}
