mod bytes;
mod il;
#[cfg(test)]
mod layout_tests;
mod names;
mod pe;
mod schema;
mod skeleton;
mod tables;
mod targets;
#[cfg(test)]
mod tests;

pub use skeleton::skeleton;

use std::collections::HashMap;

use il::CodeLocation;
use names::Names;
use pe::Image;
use schema::{
    CodedIndex, INTERFACE_IMPLEMENTATION, METHOD_DEFINITION, Row, TYPE_DEFINITION,
    TYPE_DEFINITION_OR_REFERENCE, coded_target,
};
use tables::{Cell, Heaps, Tables};
use targets::{METHOD_LIST, METHOD_NAME, Target, native_libraries, target};

const METHOD_ADDRESS: usize = 0;
const EXTENDS: usize = 3;
const IMPLEMENTER: usize = 0;
const INTERFACE: usize = 1;
const OBJECT: (&str, &str) = ("System", "Object");

pub struct Assembly {
    pub types: Vec<TypeDefinition>,
}

pub struct TypeDefinition {
    pub namespace: String,
    pub name: String,
    pub enclosing: Option<String>,
    pub supertypes: Vec<(String, String)>,
    pub methods: Vec<MethodDefinition>,
}

pub struct MethodDefinition {
    pub name: String,
    pub native_library: Option<String>,
    pub calls: Vec<Call>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Call {
    pub namespace: String,
    pub type_name: String,
    pub method: String,
    pub kind: CallKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallKind {
    Direct,
    Virtual,
    Create,
    Pointer,
}

pub fn read(bytes: &[u8]) -> Option<Assembly> {
    let (image, streams) = Image::read(bytes)?;
    let stream = |name: &str| {
        streams
            .iter()
            .find(|stream| stream.name == name)
            .map(|stream| stream.bytes)
    };
    let tables = Tables::read(Heaps {
        tables: stream("#~").or_else(|| stream("#-"))?,
        strings: stream("#Strings").unwrap_or_default(),
        blobs: stream("#Blob").unwrap_or_default(),
    })?;
    let names = Names::new(&tables);
    let libraries = native_libraries(&tables);
    let method_count = tables.rows(METHOD_DEFINITION);
    let type_count = tables.rows(TYPE_DEFINITION);
    let method_start = |row: usize| {
        if row > type_count {
            return method_count + 1;
        }
        tables
            .value(Cell {
                table: TYPE_DEFINITION,
                row,
                column: METHOD_LIST,
            })
            .map_or(method_count + 1, |start| start as usize)
    };
    let base_name = |table: usize, row: usize, column: usize| {
        let value = tables.value(Cell { table, row, column })?;
        let base = names.of(coded_target(CodedIndex {
            tables: TYPE_DEFINITION_OR_REFERENCE,
            value,
        })?)?;
        Some((base.namespace, base.name))
            .filter(|pair| (pair.0.as_str(), pair.1.as_str()) != OBJECT)
    };
    let mut supertypes: HashMap<usize, Vec<(String, String)>> = HashMap::new();
    for row in 1..=type_count {
        if let Some(base) = base_name(TYPE_DEFINITION, row, EXTENDS) {
            supertypes.entry(row).or_default().push(base);
        }
    }
    for row in 1..=tables.rows(INTERFACE_IMPLEMENTATION) {
        let implementer = tables.value(Cell {
            table: INTERFACE_IMPLEMENTATION,
            row,
            column: IMPLEMENTER,
        });
        if let (Some(implementer), Some(interface)) = (
            implementer,
            base_name(INTERFACE_IMPLEMENTATION, row, INTERFACE),
        ) {
            supertypes
                .entry(implementer as usize)
                .or_default()
                .push(interface);
        }
    }
    let mut types = Vec::with_capacity(type_count);
    for row in 1..=type_count {
        let name = names.of(Row {
            table: TYPE_DEFINITION,
            row,
        })?;
        let methods = (method_start(row)..method_start(row + 1).min(method_count + 1))
            .filter_map(|method| {
                let cell = |column| Cell {
                    table: METHOD_DEFINITION,
                    row: method,
                    column,
                };
                let address = tables.value(cell(METHOD_ADDRESS))?;
                let calls = image
                    .offset(address)
                    .and_then(|offset| il::body(CodeLocation { bytes, offset }))
                    .and_then(il::invocations)
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|invocation| {
                        target(Target {
                            tables: &tables,
                            names: &names,
                            token: invocation.token,
                        })
                        .map(|(type_name, method)| Call {
                            namespace: type_name.namespace,
                            type_name: type_name.name,
                            method,
                            kind: invocation.kind,
                        })
                    })
                    .collect();
                Some(MethodDefinition {
                    name: tables.text(cell(METHOD_NAME))?.to_string(),
                    native_library: libraries.get(&method).cloned(),
                    calls,
                })
            })
            .collect();
        types.push(TypeDefinition {
            namespace: name.namespace,
            name: name.name,
            enclosing: name.enclosing,
            supertypes: supertypes.remove(&row).unwrap_or_default(),
            methods,
        });
    }
    Some(Assembly { types })
}
