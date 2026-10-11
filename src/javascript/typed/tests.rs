use std::collections::BTreeSet;
use std::fs;

use crate::code::Rules;
use crate::deep::program::{
    AnalyzeInput, FileKey, Import, PackageFile, Program, ProgramInput, Resolver, Root, Typed,
    TypedCall, TypedCalls, TypedTarget,
};
use crate::javascript::language::JavaScript;
use crate::tree::DirectoryTree;
use crate::{Capability, Error};

const SOURCE: &str = "import { execSync } from 'child_process';\n\nfunction quiet(command: string) {\n  return command;\n}\n\nexport function main() {\n  return execSync('make');\n}\n";

struct NoPackages;

impl Resolver for NoPackages {
    fn resolve(&mut self, _import: Import) -> Result<Option<FileKey>, Error> {
        Ok(None)
    }

    fn prefetch(&mut self, _imports: &[Import]) -> Result<(), Error> {
        Ok(())
    }

    fn is_relative(&self, specifier: &str) -> bool {
        specifier.starts_with('.')
    }

    fn read(&self, _file: PackageFile) -> Option<String> {
        None
    }

    fn package_name(&self, _package: usize) -> String {
        String::new()
    }
}

fn source_file() -> FileKey {
    FileKey {
        root: Root::Repository,
        path: "src/index.ts".to_string(),
    }
}

#[test]
fn a_typed_target_that_hides_a_sink_call_does_not_hide_the_finding() {
    let folder = tempfile::tempdir().expect("folder");
    fs::create_dir_all(folder.path().join("src")).expect("src");
    fs::write(folder.path().join("src/index.ts"), SOURCE).expect("source");
    let tree = DirectoryTree::new(folder.path()).expect("tree");
    let rules = Rules::built_in().expect("rules");
    let mut resolver = NoPackages;
    let mut program = Program::new(ProgramInput {
        tree: &tree,
        rules: &rules,
        scope_of: &|_| ".".to_string(),
        language: &JavaScript,
        resolver: &mut resolver,
    });
    program.load(&["src/index.ts".to_string()]).expect("load");
    let mut types = TypedCalls::default();
    types.add(TypedCall {
        file: source_file(),
        line: 8,
        callee: "execSync".to_string(),
        typed: Typed::Targets(vec![TypedTarget {
            file: source_file(),
            name: "quiet".to_string(),
            line: 3,
        }]),
    });
    let roots: BTreeSet<String> = BTreeSet::from(["src/index.ts".to_string()]);

    let analysis = program.analyze(AnalyzeInput {
        roots: &roots,
        types: Some(&types),
    });

    let powers: Vec<Capability> = analysis
        .sightings
        .iter()
        .map(|sighting| sighting.power.capability)
        .collect();
    assert_eq!(powers, vec![Capability::Exec]);
}
