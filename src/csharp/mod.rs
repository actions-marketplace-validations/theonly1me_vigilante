mod environment;
mod language;
mod metadata;
mod nuget;
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
use environment::{CSharpEnvironment, EnvironmentInput};
use language::CSharp;
use resolver::CSharpResolver;

const BUILD_FILES: &[&str] = &[
    "packages.lock.json",
    "Directory.Build.props",
    "Directory.Packages.props",
    "nuget.config",
];

pub struct CSharpRequest<'a> {
    pub tree: &'a dyn Tree,
    pub config: &'a Config,
    pub rules: &'a Rules,
    pub scope_of: &'a dyn Fn(&str) -> String,
    pub selection: &'a Selection,
}

pub fn owns(path: &str) -> bool {
    language_of(path) == Some(SupportLang::CSharp)
}

pub fn analyze(request: CSharpRequest) -> Result<Analysis, Error> {
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
    let mut resolver = CSharpResolver {
        environment: CSharpEnvironment::new(EnvironmentInput {
            tree: request.tree,
            registry: &registry,
            sources: request.config.registry_sources(),
        })?,
    };
    let mut program = Program::new(ProgramInput {
        tree: request.tree,
        rules: request.rules,
        scope_of: request.scope_of,
        language: &CSharp,
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
