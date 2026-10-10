mod index;
mod link;
mod load;
mod parse;
mod roots;

use std::collections::HashMap;

use ast_grep_language::SupportLang;

pub use index::{Binding, Definition, Export, FileIndex, INIT, Imported, Load, Use, UseTarget};
pub use link::{LinkInput, link};
pub use roots::{RootsInput, selected_roots};

use crate::code::{Rules, SyntaxNode};
use crate::deep::graph::Owner;
use crate::tree::Tree;
use crate::{Capability, Error};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Root {
    Repository,
    Package(usize),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileKey {
    pub root: Root,
    pub path: String,
}

#[derive(Clone)]
pub struct Privilege {
    pub capability: Capability,
    pub rule: String,
}

pub struct SiteSink {
    pub definition: usize,
    pub line: usize,
    pub name: String,
    pub privilege: Privilege,
}

pub struct LoadedFile {
    pub key: FileKey,
    pub display: String,
    pub owner: Owner,
    pub index: FileIndex,
    pub privileged: HashMap<String, Vec<Privilege>>,
    pub sites: Vec<SiteSink>,
    pub resolved: HashMap<String, FileKey>,
}

#[derive(Clone, Copy)]
pub struct Import<'a> {
    pub from: &'a FileKey,
    pub specifier: &'a str,
}

pub struct PackageFile<'a> {
    pub package: usize,
    pub path: &'a str,
}

pub struct PrivilegedQuery<'a, 'r> {
    pub node: &'a SyntaxNode<'r>,
    pub is_privileged: &'a dyn Fn(&str) -> bool,
}

pub trait Language: Sync {
    fn grammar(&self) -> SupportLang;

    fn index(&self, root: &SyntaxNode) -> FileIndex;

    fn privileged_modules(&self, query: PrivilegedQuery) -> Vec<String>;

    fn stem<'p>(&self, path: &'p str) -> &'p str;

    fn imports_run_code(&self) -> bool;

    fn finds_members_by_type(&self) -> bool;
}

pub trait Resolver {
    fn resolve(&mut self, import: Import) -> Result<Option<FileKey>, Error>;

    fn prefetch(&mut self, imports: &[Import]) -> Result<(), Error>;

    fn is_relative(&self, specifier: &str) -> bool;

    fn read(&self, file: PackageFile) -> Option<String>;

    fn package_name(&self, package: usize) -> String;
}

pub struct ProgramInput<'a> {
    pub tree: &'a dyn Tree,
    pub rules: &'a Rules,
    pub scope_of: &'a dyn Fn(&str) -> String,
    pub language: &'a dyn Language,
    pub resolver: &'a mut dyn Resolver,
}

pub struct Program<'a> {
    input: ProgramInput<'a>,
    pub files: Vec<LoadedFile>,
    pub ids: HashMap<FileKey, usize>,
}
