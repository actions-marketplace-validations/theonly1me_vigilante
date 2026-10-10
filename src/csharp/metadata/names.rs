use std::collections::HashMap;

use super::bytes::Reader;
use super::schema::{
    CodedIndex, NESTED_CLASS, RESOLUTION_SCOPE, Row, TYPE_DEFINITION, TYPE_DEFINITION_OR_REFERENCE,
    TYPE_REFERENCE, TYPE_SPECIFICATION, coded_target,
};
use super::tables::{Cell, Tables};

const GENERIC_INSTANCE: u8 = 0x15;
const CLASS: u8 = 0x12;
const VALUE_TYPE: u8 = 0x11;
const NAME: usize = 1;
const NAMESPACE: usize = 2;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypeName {
    pub namespace: String,
    pub name: String,
    pub enclosing: Option<String>,
}

pub struct Names<'t, 'a> {
    tables: &'t Tables<'a>,
    enclosing_of: HashMap<usize, usize>,
}

impl<'t, 'a> Names<'t, 'a> {
    pub fn new(tables: &'t Tables<'a>) -> Self {
        let enclosing_of = (1..=tables.rows(NESTED_CLASS))
            .filter_map(|row| {
                let cell = |column| Cell {
                    table: NESTED_CLASS,
                    row,
                    column,
                };
                Some((
                    tables.value(cell(0))? as usize,
                    tables.value(cell(1))? as usize,
                ))
            })
            .collect();
        Self {
            tables,
            enclosing_of,
        }
    }

    pub fn of(&self, reference: Row) -> Option<TypeName> {
        match reference.table {
            TYPE_DEFINITION => self.definition(reference.row),
            TYPE_REFERENCE => self.reference(reference.row),
            TYPE_SPECIFICATION => self.specification(reference.row),
            _ => None,
        }
    }

    fn definition(&self, row: usize) -> Option<TypeName> {
        let (namespace, name) = self.pair(Row {
            table: TYPE_DEFINITION,
            row,
        })?;
        Some(match self.enclosing_of.get(&row) {
            Some(enclosing) => {
                let outer = self.definition(*enclosing)?;
                TypeName {
                    namespace: outer.namespace,
                    name,
                    enclosing: Some(outer.name),
                }
            }
            None => TypeName {
                namespace,
                name,
                enclosing: None,
            },
        })
    }

    fn reference(&self, row: usize) -> Option<TypeName> {
        let (namespace, name) = self.pair(Row {
            table: TYPE_REFERENCE,
            row,
        })?;
        let scope = self.tables.value(Cell {
            table: TYPE_REFERENCE,
            row,
            column: 0,
        })?;
        Some(
            match coded_target(CodedIndex {
                tables: RESOLUTION_SCOPE,
                value: scope,
            }) {
                Some(Row {
                    table: TYPE_REFERENCE,
                    row: outer_row,
                }) => {
                    let outer = self.reference(outer_row)?;
                    TypeName {
                        namespace: outer.namespace,
                        name,
                        enclosing: Some(outer.name),
                    }
                }
                _ => TypeName {
                    namespace,
                    name,
                    enclosing: None,
                },
            },
        )
    }

    fn specification(&self, row: usize) -> Option<TypeName> {
        let blob = self.tables.blob(Cell {
            table: TYPE_SPECIFICATION,
            row,
            column: 0,
        })?;
        let mut reader = Reader::new(blob);
        if reader.u8()? != GENERIC_INSTANCE || !matches!(reader.u8()?, CLASS | VALUE_TYPE) {
            return None;
        }
        let encoded = reader.compressed()?;
        let table = *TYPE_DEFINITION_OR_REFERENCE.get((encoded & 0x3) as usize)?;
        self.of(Row {
            table,
            row: (encoded >> 2) as usize,
        })
    }

    fn pair(&self, reference: Row) -> Option<(String, String)> {
        let Row { table, row } = reference;
        let text = |column| {
            self.tables
                .text(Cell { table, row, column })
                .map(str::to_string)
        };
        let name = text(NAME)?;
        let name = name.split('`').next().unwrap_or(&name).to_string();
        Some((text(NAMESPACE).unwrap_or_default(), name))
    }
}
