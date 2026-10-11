mod environment;
mod language;
mod prefetch;
mod resolver;
mod standard_library;
mod syntax;

use std::collections::BTreeMap;

use ast_grep_language::SupportLang;

use crate::Error;
use crate::code::{Rules, language_of};
use crate::config::Config;
use crate::deep::Analysis;
use crate::deep::program::{AnalyzeInput, Program, ProgramInput, RootsInput, selected_roots};
use crate::lockfiles::python::PYTHON_LOCKFILES;
use crate::registry::Registry;
use crate::scan::Selection;
use crate::tree::Tree;
use environment::{Environment, EnvironmentInput};
use language::Python;
use resolver::PythonResolver;

pub struct PythonRequest<'a> {
    pub tree: &'a dyn Tree,
    pub config: &'a Config,
    pub rules: &'a Rules,
    pub scope_of: &'a dyn Fn(&str) -> String,
    pub selection: &'a Selection,
}

pub fn owns(path: &str) -> bool {
    language_of(path) == Some(SupportLang::Python)
}

pub fn analyze(request: PythonRequest) -> Result<Analysis, Error> {
    let roots = selected_roots(RootsInput {
        tree: request.tree,
        config: request.config,
        scope_of: request.scope_of,
        selection: request.selection,
        is_code: is_code_file,
        lockfiles: PYTHON_LOCKFILES,
    });
    if roots.is_empty() {
        return Ok(Analysis::default());
    }
    let registry = Registry::new(Registry::default_cache_root());
    let mut resolver = PythonResolver {
        environment: Environment::new(EnvironmentInput {
            tree: request.tree,
            registry: &registry,
            sources: request.config.registry_sources(),
        }),
        tree: request.tree,
        scope_of: request.scope_of,
        unresolved: BTreeMap::new(),
    };
    let mut program = Program::new(ProgramInput {
        tree: request.tree,
        rules: request.rules,
        scope_of: request.scope_of,
        language: &Python,
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

fn is_code_file(path: &str) -> bool {
    path.ends_with(".py")
}
