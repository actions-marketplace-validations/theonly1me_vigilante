use super::CallKind;
use super::bytes::Reader;

const TINY_HEADER: u8 = 0x2;
const FAT_HEADER: u8 = 0x3;
const HEADER_KIND_MASK: u8 = 0x3;
const JUMP: u8 = 0x27;
const CALL: u8 = 0x28;
const CALL_VIRTUAL: u8 = 0x6F;
const NEW_OBJECT: u8 = 0x73;
const SWITCH: u8 = 0x45;
const TWO_BYTE_PREFIX: u8 = 0xFE;
const LOAD_FUNCTION: u8 = 0x06;
const LOAD_VIRTUAL_FUNCTION: u8 = 0x07;

pub struct Invocation {
    pub token: u32,
    pub kind: CallKind,
}

pub struct CodeLocation<'a> {
    pub bytes: &'a [u8],
    pub offset: usize,
}

pub fn body(location: CodeLocation<'_>) -> Option<&[u8]> {
    let mut reader = Reader::new(location.bytes).at(location.offset)?;
    let first = reader.u8()?;
    match first & HEADER_KIND_MASK {
        TINY_HEADER => reader.take(usize::from(first >> 2)),
        FAT_HEADER => {
            reader.skip(3)?;
            let size = reader.u32()? as usize;
            reader.skip(4)?;
            reader.take(size)
        }
        _ => None,
    }
}

pub fn invocations(code: &[u8]) -> Option<Vec<Invocation>> {
    let mut reader = Reader::new(code);
    let mut found = Vec::new();
    while !reader.is_done() {
        let opcode = reader.u8()?;
        if opcode == TWO_BYTE_PREFIX {
            let second = reader.u8()?;
            let kind = match second {
                LOAD_FUNCTION | LOAD_VIRTUAL_FUNCTION => Some(CallKind::Pointer),
                _ => None,
            };
            if let Some(kind) = kind {
                found.push(Invocation {
                    token: reader.u32()?,
                    kind,
                });
            } else {
                reader.skip(two_byte_operand(second))?;
            }
            continue;
        }
        let kind = match opcode {
            CALL | JUMP => Some(CallKind::Direct),
            CALL_VIRTUAL => Some(CallKind::Virtual),
            NEW_OBJECT => Some(CallKind::Create),
            _ => None,
        };
        if let Some(kind) = kind {
            found.push(Invocation {
                token: reader.u32()?,
                kind,
            });
        } else if opcode == SWITCH {
            let targets = reader.u32()? as usize;
            reader.skip(targets.checked_mul(4)?)?;
        } else {
            reader.skip(one_byte_operand(opcode))?;
        }
    }
    Some(found)
}

fn one_byte_operand(opcode: u8) -> usize {
    match opcode {
        0x0E..=0x13 | 0x1F | 0x2B..=0x37 | 0xDE => 1,
        0x21 | 0x23 => 8,
        0x20 | 0x22 | 0x29 | 0x38..=0x44 | 0x70..=0x72 | 0x74 | 0x75 | 0x79 | 0x7B..=0x81 => 4,
        0x8C | 0x8D | 0x8F | 0xA3..=0xA5 | 0xC2 | 0xC6 | 0xD0 | 0xDD => 4,
        _ => 0,
    }
}

fn two_byte_operand(opcode: u8) -> usize {
    match opcode {
        0x15 | 0x16 | 0x1C => 4,
        0x09..=0x0E => 2,
        0x12 | 0x19 => 1,
        _ => 0,
    }
}
