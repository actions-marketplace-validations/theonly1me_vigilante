mod language;
mod manifest;
mod packages;
mod paths;
mod prefetch;
mod projects;
mod resolver;
mod syntax;

use ast_grep_language::SupportLang;

use crate::Error;
use crate::code::{Rules, language_of};
use crate::config::Config;
use crate::deep::Analysis;
use crate::deep::program::{LinkInput, Program, ProgramInput, RootsInput, link, selected_roots};
use crate::lockfiles::JAVASCRIPT_LOCKFILES;
use crate::registry::Registry;
use crate::scan::Selection;
use crate::tree::Tree;
use language::JavaScript;
use packages::{Packages, PackagesInput};
use paths::is_code_file;
use projects::Projects;
use resolver::JavaScriptResolver;

pub struct JavaScriptRequest<'a> {
    pub tree: &'a dyn Tree,
    pub config: &'a Config,
    pub rules: &'a Rules,
    pub scope_of: &'a dyn Fn(&str) -> String,
    pub selection: &'a Selection,
}

pub fn owns(path: &str) -> bool {
    language_of(path) == Some(SupportLang::Tsx)
}

pub fn analyze(request: JavaScriptRequest) -> Result<Analysis, Error> {
    let roots = selected_roots(RootsInput {
        tree: request.tree,
        config: request.config,
        scope_of: request.scope_of,
        selection: request.selection,
        is_code: is_code_file,
        lockfiles: JAVASCRIPT_LOCKFILES,
    });
    if roots.is_empty() {
        return Ok(Analysis::default());
    }
    let registry = Registry::new(Registry::default_cache_root());
    let mut resolver = JavaScriptResolver {
        packages: Packages::new(PackagesInput {
            projects: Projects::new(request.tree),
            registry: &registry,
            sources: request.config.registry_sources(),
        }),
        scope_of: request.scope_of,
    };
    let mut program = Program::new(ProgramInput {
        tree: request.tree,
        rules: request.rules,
        scope_of: request.scope_of,
        language: &JavaScript,
        resolver: &mut resolver,
    });
    let root_paths: Vec<String> = roots.iter().cloned().collect();
    program.load(&root_paths)?;
    let linked = link(LinkInput {
        program: &program,
        roots: &roots,
    });
    drop(program);
    let mut analysis = linked.graph.analyze(&linked.scopes);
    analysis.skipped = resolver.packages.into_skipped();
    Ok(analysis)
}
