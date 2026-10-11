use super::project::{PACKAGE_PREFIX, Project};
use crate::deep::program::{Program, TypedCalls, TypedTarget};
use crate::scip::{CallsInput, Index, typed_calls, without_call_suffix};

const SYMBOL_PARTS: usize = 5;

pub struct Indexed<'a, 'p> {
    pub index: &'a Index,
    pub project: &'a Project,
    pub program: &'a Program<'p>,
}

pub fn python_calls(indexed: Indexed) -> TypedCalls {
    let Indexed {
        index,
        project,
        program,
    } = indexed;
    typed_calls(CallsInput {
        index,
        keys: &project.keys,
        external: &|symbol| {
            package_target(PackageSymbol {
                symbol,
                project,
                program,
            })
        },
    })
}

struct PackageSymbol<'a, 'p> {
    symbol: &'a str,
    project: &'a Project,
    program: &'a Program<'p>,
}

fn package_target(wanted: PackageSymbol) -> Option<TypedTarget> {
    let PackageSymbol {
        symbol,
        project,
        program,
    } = wanted;
    let parts: Vec<&str> = symbol.splitn(SYMBOL_PARTS, ' ').collect();
    let [_, _, package, _, descriptors] = parts[..] else {
        return None;
    };
    let package: usize = package.strip_prefix(PACKAGE_PREFIX)?.parse().ok()?;
    let (module, member) = descriptors.split_once('/')?;
    let module = module.trim_matches('`').replace('.', "/");
    let qualified = without_call_suffix(member)?.replace('#', ".");
    if !qualified
        .chars()
        .all(|character| character.is_alphanumeric() || character == '_' || character == '.')
    {
        return None;
    }
    let file = [format!("{module}.py"), format!("{module}/__init__.py")]
        .into_iter()
        .find_map(|path| project.modules.get(&(package, path)))?;
    let loaded = program.files.iter().find(|loaded| &loaded.key == file)?;
    let definition = loaded
        .index
        .definitions
        .iter()
        .skip(1)
        .find(|definition| definition.name == qualified)?;
    Some(TypedTarget {
        file: file.clone(),
        name: definition.simple.clone(),
        line: loaded.line_of(definition.range.start),
    })
}
