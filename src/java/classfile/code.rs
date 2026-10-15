use super::bytes::Reader;
use super::constants::{Constant, ConstantPool, MethodReference};
use super::{Call, CallKind};

const INVOKE_VIRTUAL: u8 = 0xb6;
const INVOKE_SPECIAL: u8 = 0xb7;
const INVOKE_STATIC: u8 = 0xb8;
const INVOKE_INTERFACE: u8 = 0xb9;
const INVOKE_DYNAMIC: u8 = 0xba;
const TABLE_SWITCH: u8 = 0xaa;
const LOOKUP_SWITCH: u8 = 0xab;
const WIDE: u8 = 0xc4;
const INCREMENT: u8 = 0x84;
const OPERAND_BYTES: &[(u8, u8, usize)] = &[
    (0x10, 0x10, 1),
    (0x11, 0x11, 2),
    (0x12, 0x12, 1),
    (0x13, 0x14, 2),
    (0x15, 0x19, 1),
    (0x36, 0x3a, 1),
    (0x84, 0x84, 2),
    (0x99, 0xa8, 2),
    (0xa9, 0xa9, 1),
    (0xb2, 0xb8, 2),
    (0xb9, 0xba, 4),
    (0xbb, 0xbb, 2),
    (0xbc, 0xbc, 1),
    (0xbd, 0xbd, 2),
    (0xc0, 0xc1, 2),
    (0xc5, 0xc5, 3),
    (0xc6, 0xc7, 2),
    (0xc8, 0xc9, 4),
];

pub struct CodeRequest<'a> {
    pub pool: &'a ConstantPool,
    pub attribute: &'a [u8],
    pub bootstrap: &'a [Vec<Call>],
}

pub fn calls(request: CodeRequest) -> Option<Vec<Call>> {
    let mut header = Reader::new(request.attribute);
    header.skip(4)?;
    let length = header.u32()? as usize;
    let code = header.bytes(length)?;
    let mut reader = Reader::new(code);
    let mut calls = Vec::new();
    while !reader.is_done() {
        let start = reader.position();
        let opcode = reader.u8()?;
        let kind = match opcode {
            INVOKE_VIRTUAL => Some(CallKind::Virtual),
            INVOKE_SPECIAL => Some(CallKind::Special),
            INVOKE_STATIC => Some(CallKind::Static),
            INVOKE_INTERFACE => Some(CallKind::Interface),
            _ => None,
        };
        if let Some(kind) = kind {
            let index = reader.u16()?;
            if opcode == INVOKE_INTERFACE {
                reader.skip(2)?;
            }
            calls.extend(request.pool.method_ref(MethodReference { index, kind }));
            continue;
        }
        if opcode == INVOKE_DYNAMIC {
            let index = reader.u16()?;
            reader.skip(2)?;
            if let Some(Constant::Dynamic { bootstrap }) = request.pool.get(index)
                && let Some(targets) = request.bootstrap.get(usize::from(*bootstrap))
            {
                calls.extend(targets.iter().cloned());
            }
            continue;
        }
        let operands = match opcode {
            TABLE_SWITCH => table_switch(&mut Switch {
                reader: &mut reader,
                start,
            })?,
            LOOKUP_SWITCH => lookup_switch(&mut Switch {
                reader: &mut reader,
                start,
            })?,
            WIDE => {
                if reader.u8()? == INCREMENT {
                    4
                } else {
                    2
                }
            }
            other => OPERAND_BYTES
                .iter()
                .find(|(first, last, _)| (*first..=*last).contains(&other))
                .map_or(0, |(_, _, count)| *count),
        };
        reader.skip(operands)?;
    }
    Some(calls)
}

struct Switch<'a, 'r> {
    reader: &'a mut Reader<'r>,
    start: usize,
}

fn table_switch(switch: &mut Switch) -> Option<usize> {
    skip_padding(switch)?;
    switch.reader.skip(4)?;
    let low = switch.reader.i32()?;
    let high = switch.reader.i32()?;
    let count = usize::try_from(i64::from(high) - i64::from(low) + 1).ok()?;
    count.checked_mul(4)
}

fn lookup_switch(switch: &mut Switch) -> Option<usize> {
    skip_padding(switch)?;
    switch.reader.skip(4)?;
    let pairs = usize::try_from(switch.reader.i32()?).ok()?;
    pairs.checked_mul(8)
}

fn skip_padding(switch: &mut Switch) -> Option<()> {
    let padding = (4 - (switch.start + 1) % 4) % 4;
    switch.reader.skip(padding)
}
