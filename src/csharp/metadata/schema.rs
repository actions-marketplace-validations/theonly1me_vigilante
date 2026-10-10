pub const TABLE_COUNT: usize = 64;

pub const MODULE: usize = 0x00;
pub const TYPE_REFERENCE: usize = 0x01;
pub const TYPE_DEFINITION: usize = 0x02;
pub const FIELD: usize = 0x04;
pub const METHOD_DEFINITION: usize = 0x06;
pub const PARAMETER: usize = 0x08;
pub const INTERFACE_IMPLEMENTATION: usize = 0x09;
pub const MEMBER_REFERENCE: usize = 0x0A;
pub const DECLARATIVE_SECURITY: usize = 0x0E;
pub const STANDALONE_SIGNATURE: usize = 0x11;
pub const EVENT: usize = 0x14;
pub const PROPERTY: usize = 0x17;
pub const MODULE_REFERENCE: usize = 0x1A;
pub const TYPE_SPECIFICATION: usize = 0x1B;
pub const IMPLEMENTATION_MAP: usize = 0x1C;
pub const ASSEMBLY: usize = 0x20;
pub const ASSEMBLY_REFERENCE: usize = 0x23;
pub const FILE: usize = 0x26;
pub const EXPORTED_TYPE: usize = 0x27;
pub const MANIFEST_RESOURCE: usize = 0x28;
pub const NESTED_CLASS: usize = 0x29;
pub const GENERIC_PARAMETER: usize = 0x2A;
pub const METHOD_SPECIFICATION: usize = 0x2B;
pub const GENERIC_PARAMETER_CONSTRAINT: usize = 0x2C;

const NONE: usize = usize::MAX;

#[derive(Clone, Copy)]
pub enum Column {
    Fixed(usize),
    Strings,
    Guid,
    Blob,
    Table(usize),
    Coded(&'static [usize]),
}

pub const TYPE_DEFINITION_OR_REFERENCE: &[usize] =
    &[TYPE_DEFINITION, TYPE_REFERENCE, TYPE_SPECIFICATION];
pub const MEMBER_REFERENCE_PARENT: &[usize] = &[
    TYPE_DEFINITION,
    TYPE_REFERENCE,
    MODULE_REFERENCE,
    METHOD_DEFINITION,
    TYPE_SPECIFICATION,
];
pub const METHOD_DEFINITION_OR_REFERENCE: &[usize] = &[METHOD_DEFINITION, MEMBER_REFERENCE];
pub const MEMBER_FORWARDED: &[usize] = &[FIELD, METHOD_DEFINITION];
pub const RESOLUTION_SCOPE: &[usize] =
    &[MODULE, MODULE_REFERENCE, ASSEMBLY_REFERENCE, TYPE_REFERENCE];
const HAS_CONSTANT: &[usize] = &[FIELD, PARAMETER, PROPERTY];
const HAS_CUSTOM_ATTRIBUTE: &[usize] = &[
    METHOD_DEFINITION,
    FIELD,
    TYPE_REFERENCE,
    TYPE_DEFINITION,
    PARAMETER,
    INTERFACE_IMPLEMENTATION,
    MEMBER_REFERENCE,
    MODULE,
    DECLARATIVE_SECURITY,
    PROPERTY,
    EVENT,
    STANDALONE_SIGNATURE,
    MODULE_REFERENCE,
    TYPE_SPECIFICATION,
    ASSEMBLY,
    ASSEMBLY_REFERENCE,
    FILE,
    EXPORTED_TYPE,
    MANIFEST_RESOURCE,
    GENERIC_PARAMETER,
    GENERIC_PARAMETER_CONSTRAINT,
    METHOD_SPECIFICATION,
];
const HAS_FIELD_MARSHAL: &[usize] = &[FIELD, PARAMETER];
const HAS_DECLARATIVE_SECURITY: &[usize] = &[TYPE_DEFINITION, METHOD_DEFINITION, ASSEMBLY];
const HAS_SEMANTICS: &[usize] = &[EVENT, PROPERTY];
const IMPLEMENTATION: &[usize] = &[FILE, ASSEMBLY_REFERENCE, EXPORTED_TYPE];
const CUSTOM_ATTRIBUTE_TYPE: &[usize] = &[NONE, NONE, METHOD_DEFINITION, MEMBER_REFERENCE, NONE];
const TYPE_OR_METHOD_DEFINITION: &[usize] = &[TYPE_DEFINITION, METHOD_DEFINITION];

use Column::{Blob, Coded, Fixed, Guid, Strings, Table};

pub fn columns(table: usize) -> &'static [Column] {
    match table {
        0x00 => &[Fixed(2), Strings, Guid, Guid, Guid],
        0x01 => &[Coded(RESOLUTION_SCOPE), Strings, Strings],
        0x02 => &[
            Fixed(4),
            Strings,
            Strings,
            Coded(TYPE_DEFINITION_OR_REFERENCE),
            Table(FIELD),
            Table(METHOD_DEFINITION),
        ],
        0x03 => &[Table(FIELD)],
        0x04 => &[Fixed(2), Strings, Blob],
        0x05 => &[Table(METHOD_DEFINITION)],
        0x06 => &[
            Fixed(4),
            Fixed(2),
            Fixed(2),
            Strings,
            Blob,
            Table(PARAMETER),
        ],
        0x07 => &[Table(PARAMETER)],
        0x08 => &[Fixed(2), Fixed(2), Strings],
        0x09 => &[Table(TYPE_DEFINITION), Coded(TYPE_DEFINITION_OR_REFERENCE)],
        0x0A => &[Coded(MEMBER_REFERENCE_PARENT), Strings, Blob],
        0x0B => &[Fixed(2), Coded(HAS_CONSTANT), Blob],
        0x0C => &[
            Coded(HAS_CUSTOM_ATTRIBUTE),
            Coded(CUSTOM_ATTRIBUTE_TYPE),
            Blob,
        ],
        0x0D => &[Coded(HAS_FIELD_MARSHAL), Blob],
        0x0E => &[Fixed(2), Coded(HAS_DECLARATIVE_SECURITY), Blob],
        0x0F => &[Fixed(2), Fixed(4), Table(TYPE_DEFINITION)],
        0x10 => &[Fixed(4), Table(FIELD)],
        0x11 => &[Blob],
        0x12 => &[Table(TYPE_DEFINITION), Table(EVENT)],
        0x13 => &[Table(EVENT)],
        0x14 => &[Fixed(2), Strings, Coded(TYPE_DEFINITION_OR_REFERENCE)],
        0x15 => &[Table(TYPE_DEFINITION), Table(PROPERTY)],
        0x16 => &[Table(PROPERTY)],
        0x17 => &[Fixed(2), Strings, Blob],
        0x18 => &[Fixed(2), Table(METHOD_DEFINITION), Coded(HAS_SEMANTICS)],
        0x19 => &[
            Table(TYPE_DEFINITION),
            Coded(METHOD_DEFINITION_OR_REFERENCE),
            Coded(METHOD_DEFINITION_OR_REFERENCE),
        ],
        0x1A => &[Strings],
        0x1B => &[Blob],
        0x1C => &[
            Fixed(2),
            Coded(MEMBER_FORWARDED),
            Strings,
            Table(MODULE_REFERENCE),
        ],
        0x1D => &[Fixed(4), Table(FIELD)],
        0x1E => &[Fixed(4), Fixed(4)],
        0x1F => &[Fixed(4)],
        0x20 => &[Fixed(4), Fixed(8), Fixed(4), Blob, Strings, Strings],
        0x21 => &[Fixed(4)],
        0x22 => &[Fixed(4), Fixed(4), Fixed(4)],
        0x23 => &[Fixed(8), Fixed(4), Blob, Strings, Strings, Blob],
        0x24 => &[Fixed(4), Table(ASSEMBLY_REFERENCE)],
        0x25 => &[Fixed(4), Fixed(4), Fixed(4), Table(ASSEMBLY_REFERENCE)],
        0x26 => &[Fixed(4), Strings, Blob],
        0x27 => &[Fixed(4), Fixed(4), Strings, Strings, Coded(IMPLEMENTATION)],
        0x28 => &[Fixed(4), Fixed(4), Strings, Coded(IMPLEMENTATION)],
        0x29 => &[Table(TYPE_DEFINITION), Table(TYPE_DEFINITION)],
        0x2A => &[
            Fixed(2),
            Fixed(2),
            Coded(TYPE_OR_METHOD_DEFINITION),
            Strings,
        ],
        0x2B => &[Coded(METHOD_DEFINITION_OR_REFERENCE), Blob],
        0x2C => &[
            Table(GENERIC_PARAMETER),
            Coded(TYPE_DEFINITION_OR_REFERENCE),
        ],
        _ => &[],
    }
}

pub fn tag_bits(tables: &[usize]) -> u32 {
    usize::BITS - (tables.len() - 1).leading_zeros()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Row {
    pub table: usize,
    pub row: usize,
}

pub struct CodedIndex<'a> {
    pub tables: &'a [usize],
    pub value: u32,
}

pub fn coded_target(coded: CodedIndex) -> Option<Row> {
    let CodedIndex { tables, value } = coded;
    let bits = tag_bits(tables);
    let table = *tables.get((value & ((1 << bits) - 1)) as usize)?;
    let row = (value >> bits) as usize;
    (table != NONE && row > 0).then_some(Row { table, row })
}
