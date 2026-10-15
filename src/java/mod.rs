mod classfile;
mod environment;
mod gradle;
mod language;
mod maven;
mod resolver;
mod syntax;

use ast_grep_language::SupportLang;

use crate::Error;
use crate::code::{Rules, language_of};
use crate::config::Config;
use crate::deep::Analysis;
use crate::deep::program::{AnalyzeInput, Program, ProgramInput, RootsInput, selected_roots};
use crate::registry::Registry;
use crate::scan::Selection;
use crate::tree::Tree;
use environment::{EnvironmentInput, JavaEnvironment};
use language::Java;
use resolver::{JavaResolver, ResolverInput};

const BUILD_FILES: &[&str] = &[
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    "gradle.lockfile",
];

pub struct JavaRequest<'a> {
    pub tree: &'a dyn Tree,
    pub config: &'a Config,
    pub rules: &'a Rules,
    pub scope_of: &'a dyn Fn(&str) -> String,
    pub selection: &'a Selection,
}

pub fn owns(path: &str) -> bool {
    language_of(path) == Some(SupportLang::Java)
}

pub fn analyze(request: JavaRequest) -> Result<Analysis, Error> {
    let roots = selected_roots(RootsInput {
        tree: request.tree,
        config: request.config,
        scope_of: request.scope_of,
        selection: request.selection,
        is_code: is_code_file,
        lockfiles: BUILD_FILES,
    });
    if roots.is_empty() {
        return Ok(Analysis::default());
    }
    let registry = Registry::new(Registry::default_cache_root());
    let environment = JavaEnvironment::new(EnvironmentInput {
        tree: request.tree,
        registry: &registry,
        sources: request.config.registry_sources(),
    });
    let mut resolver = JavaResolver::new(ResolverInput {
        environment,
        tree: request.tree,
    });
    let mut program = Program::new(ProgramInput {
        tree: request.tree,
        rules: request.rules,
        scope_of: request.scope_of,
        language: &Java,
        resolver: &mut resolver,
    });
    let root_paths: Vec<String> = roots.iter().cloned().collect();
    program.load(&root_paths)?;
    let mut analysis = program.analyze(AnalyzeInput {
        roots: &roots,
        check: None,
    });
    drop(program);
    analysis.skipped.extend(resolver.into_skipped());
    Ok(analysis)
}

fn is_code_file(path: &str) -> bool {
    path.ends_with(".java")
}
