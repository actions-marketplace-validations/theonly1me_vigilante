use super::bytes::Reader;
use super::{Call, CallKind};

#[derive(Debug)]
pub enum Constant {
    Utf8(String),
    Class { name: u16 },
    Member { class: u16, name_and_type: u16 },
    NameAndType { name: u16 },
    MethodHandle { reference: u16 },
    Dynamic { bootstrap: u16 },
    Other,
}

pub struct MethodReference {
    pub index: u16,
    pub kind: CallKind,
}

pub struct ConstantPool {
    entries: Vec<Option<Constant>>,
}

impl ConstantPool {
    pub fn read(reader: &mut Reader) -> Option<Self> {
        let count = usize::from(reader.u16()?);
        let mut entries: Vec<Option<Constant>> = Vec::with_capacity(count);
        entries.push(None);
        while entries.len() < count {
            let tag = reader.u8()?;
            let (entry, slots) = match tag {
                1 => {
                    let length = usize::from(reader.u16()?);
                    let text = String::from_utf8_lossy(reader.bytes(length)?).into_owned();
                    (Constant::Utf8(text), 1)
                }
                3 | 4 => {
                    reader.skip(4)?;
                    (Constant::Other, 1)
                }
                5 | 6 => {
                    reader.skip(8)?;
                    (Constant::Other, 2)
                }
                7 => (
                    Constant::Class {
                        name: reader.u16()?,
                    },
                    1,
                ),
                8 | 16 | 19 | 20 => {
                    reader.skip(2)?;
                    (Constant::Other, 1)
                }
                9..=11 => (
                    Constant::Member {
                        class: reader.u16()?,
                        name_and_type: reader.u16()?,
                    },
                    1,
                ),
                12 => {
                    let name = reader.u16()?;
                    reader.skip(2)?;
                    (Constant::NameAndType { name }, 1)
                }
                15 => {
                    reader.skip(1)?;
                    (
                        Constant::MethodHandle {
                            reference: reader.u16()?,
                        },
                        1,
                    )
                }
                17 | 18 => {
                    let bootstrap = reader.u16()?;
                    reader.skip(2)?;
                    (Constant::Dynamic { bootstrap }, 1)
                }
                _ => return None,
            };
            entries.push(Some(entry));
            for _ in 1..slots {
                entries.push(None);
            }
        }
        Some(Self { entries })
    }

    pub fn get(&self, index: u16) -> Option<&Constant> {
        self.entries.get(usize::from(index))?.as_ref()
    }

    pub fn utf8(&self, index: u16) -> Option<&str> {
        match self.get(index)? {
            Constant::Utf8(text) => Some(text),
            _ => None,
        }
    }

    pub fn class_name(&self, index: u16) -> Option<&str> {
        match self.get(index)? {
            Constant::Class { name } => self.utf8(*name),
            _ => None,
        }
    }

    pub fn method_ref(&self, reference: MethodReference) -> Option<Call> {
        let Constant::Member {
            class,
            name_and_type,
        } = self.get(reference.index)?
        else {
            return None;
        };
        let Constant::NameAndType { name } = self.get(*name_and_type)? else {
            return None;
        };
        Some(Call {
            class: self.class_name(*class)?.to_string(),
            name: self.utf8(*name)?.to_string(),
            kind: reference.kind,
        })
    }
}
