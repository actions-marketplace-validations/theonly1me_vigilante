mod bundler;
mod environment;
mod language;
mod names;
mod resolver;
mod search;
mod standard_library;
mod syntax;

use std::collections::{BTreeMap, HashMap};

use ast_grep_language::SupportLang;

use crate::Error;
use crate::code::{Rules, language_of};
use crate::config::Config;
use crate::deep::Analysis;
use crate::deep::program::{AnalyzeInput, Program, ProgramInput, RootsInput, selected_roots};
use crate::registry::Registry;
use crate::scan::Selection;
use crate::tree::Tree;
use environment::{EnvironmentInput, RubyEnvironment};
use language::Ruby;
use resolver::RubyResolver;

const BUILD_FILES: &[&str] = &["Gemfile", "Gemfile.lock", "gems.rb", "gems.locked"];
const SOURCE_SUFFIX: &str = ".rb";

pub struct RubyRequest<'a> {
    pub tree: &'a dyn Tree,
    pub config: &'a Config,
    pub rules: &'a Rules,
    pub scope_of: &'a dyn Fn(&str) -> String,
    pub selection: &'a Selection,
}

pub fn owns(path: &str) -> bool {
    language_of(path) == Some(SupportLang::Ruby)
}

pub fn analyze(request: RubyRequest) -> Result<Analysis, Error> {
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
    let mut own_by_name: HashMap<String, Vec<String>> = HashMap::new();
    for path in request.tree.paths() {
        if path.ends_with(SOURCE_SUFFIX) {
            let name = path.rsplit('/').next().unwrap_or(path).to_string();
            own_by_name.entry(name).or_default().push(path.clone());
        }
    }
    let mut resolver = RubyResolver {
        environment: RubyEnvironment::new(EnvironmentInput {
            tree: request.tree,
            registry: &registry,
            sources: request.config.registry_sources(),
        }),
        scope_of: request.scope_of,
        unresolved: BTreeMap::new(),
        own_by_name,
    };
    let mut program = Program::new(ProgramInput {
        tree: request.tree,
        rules: request.rules,
        scope_of: request.scope_of,
        language: &Ruby,
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
