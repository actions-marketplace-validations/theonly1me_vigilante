use std::collections::HashMap;

use super::names::{Names, TypeName};
use super::schema::{
    CodedIndex, IMPLEMENTATION_MAP, MEMBER_FORWARDED, MEMBER_REFERENCE, MEMBER_REFERENCE_PARENT,
    METHOD_DEFINITION, METHOD_DEFINITION_OR_REFERENCE, METHOD_SPECIFICATION, MODULE_REFERENCE, Row,
    TYPE_DEFINITION, coded_target,
};
use super::tables::{Cell, Tables};

pub const METHOD_LIST: usize = 5;
pub const METHOD_NAME: usize = 3;
const MEMBER_PARENT: usize = 0;
const MEMBER_NAME: usize = 1;
const TOKEN_TABLE_SHIFT: u32 = 24;
const TOKEN_ROW_MASK: u32 = 0x00FF_FFFF;

pub struct Target<'t, 'n, 'a> {
    pub tables: &'t Tables<'a>,
    pub names: &'n Names<'t, 'a>,
    pub token: u32,
}

pub fn target(wanted: Target) -> Option<(TypeName, String)> {
    let Target {
        tables,
        names,
        token,
    } = wanted;
    let row = (token & TOKEN_ROW_MASK) as usize;
    match (token >> TOKEN_TABLE_SHIFT) as usize {
        METHOD_DEFINITION => {
            let owner = owner_of(OwnerQuery {
                tables,
                method: row,
            })?;
            Some((
                names.of(Row {
                    table: TYPE_DEFINITION,
                    row: owner,
                })?,
                tables
                    .text(Cell {
                        table: METHOD_DEFINITION,
                        row,
                        column: METHOD_NAME,
                    })?
                    .to_string(),
            ))
        }
        MEMBER_REFERENCE => {
            let cell = |column| Cell {
                table: MEMBER_REFERENCE,
                row,
                column,
            };
            let parent = coded_target(CodedIndex {
                tables: MEMBER_REFERENCE_PARENT,
                value: tables.value(cell(MEMBER_PARENT))?,
            })?;
            Some((
                names.of(parent)?,
                tables.text(cell(MEMBER_NAME))?.to_string(),
            ))
        }
        METHOD_SPECIFICATION => {
            let method = tables.value(Cell {
                table: METHOD_SPECIFICATION,
                row,
                column: 0,
            })?;
            let inner = coded_target(CodedIndex {
                tables: METHOD_DEFINITION_OR_REFERENCE,
                value: method,
            })?;
            target(Target {
                tables,
                names,
                token: ((inner.table as u32) << TOKEN_TABLE_SHIFT) | inner.row as u32,
            })
        }
        _ => None,
    }
}

struct OwnerQuery<'t, 'a> {
    tables: &'t Tables<'a>,
    method: usize,
}

fn owner_of(query: OwnerQuery) -> Option<usize> {
    let OwnerQuery { tables, method } = query;
    (1..=tables.rows(TYPE_DEFINITION)).rev().find(|row| {
        tables
            .value(Cell {
                table: TYPE_DEFINITION,
                row: *row,
                column: METHOD_LIST,
            })
            .is_some_and(|start| start as usize <= method)
    })
}

pub fn native_libraries(tables: &Tables) -> HashMap<usize, String> {
    (1..=tables.rows(IMPLEMENTATION_MAP))
        .filter_map(|row| {
            let cell = |column| Cell {
                table: IMPLEMENTATION_MAP,
                row,
                column,
            };
            let forwarded = coded_target(CodedIndex {
                tables: MEMBER_FORWARDED,
                value: tables.value(cell(1))?,
            })?;
            let library = tables.text(Cell {
                table: MODULE_REFERENCE,
                row: tables.value(cell(3))? as usize,
                column: 0,
            })?;
            (forwarded.table == METHOD_DEFINITION).then(|| (forwarded.row, library.to_string()))
        })
        .collect()
}
