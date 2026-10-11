#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Eq)]
pub struct Index {
    pub documents: Vec<Document>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Document {
    pub path: String,
    pub occurrences: Vec<Occurrence>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Occurrence {
    pub line: usize,
    pub column: usize,
    pub symbol: String,
    pub is_definition: bool,
}

const DEFINITION_ROLE: u64 = 1;

enum Value<'a> {
    Number(u64),
    Bytes(&'a [u8]),
    Fixed,
}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn varint(&mut self) -> Option<u64> {
        let mut value = 0u64;
        for shift in (0..64).step_by(7) {
            let byte = *self.bytes.get(self.position)?;
            self.position += 1;
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Some(value);
            }
        }
        None
    }

    fn take(&mut self, length: usize) -> Option<&'a [u8]> {
        let end = self.position.checked_add(length)?;
        let taken = self.bytes.get(self.position..end)?;
        self.position = end;
        Some(taken)
    }

    fn fields(mut self) -> Option<Vec<(u64, Value<'a>)>> {
        let mut fields = Vec::new();
        while self.position < self.bytes.len() {
            let tag = self.varint()?;
            let value = match tag & 7 {
                0 => Value::Number(self.varint()?),
                1 => self.take(8).map(|_| Value::Fixed)?,
                2 => {
                    let length = usize::try_from(self.varint()?).ok()?;
                    Value::Bytes(self.take(length)?)
                }
                5 => self.take(4).map(|_| Value::Fixed)?,
                _ => return None,
            };
            fields.push((tag >> 3, value));
        }
        Some(fields)
    }
}

pub fn read_index(bytes: &[u8]) -> Option<Index> {
    let mut documents = Vec::new();
    for (number, value) in Reader::new(bytes).fields()? {
        if let (2, Value::Bytes(document)) = (number, value) {
            documents.push(read_document(document)?);
        }
    }
    Some(Index { documents })
}

fn read_document(bytes: &[u8]) -> Option<Document> {
    let mut document = Document {
        path: String::new(),
        occurrences: Vec::new(),
    };
    for (number, value) in Reader::new(bytes).fields()? {
        match (number, value) {
            (1, Value::Bytes(path)) => document.path = String::from_utf8(path.to_vec()).ok()?,
            (2, Value::Bytes(occurrence)) => {
                document.occurrences.push(read_occurrence(occurrence)?)
            }
            _ => {}
        }
    }
    Some(document)
}

fn read_occurrence(bytes: &[u8]) -> Option<Occurrence> {
    let mut range = Vec::new();
    let mut symbol = String::new();
    let mut roles = 0;
    for (number, value) in Reader::new(bytes).fields()? {
        match (number, value) {
            (1, Value::Bytes(packed)) => {
                let mut reader = Reader::new(packed);
                while reader.position < packed.len() {
                    range.push(reader.varint()?);
                }
            }
            (1, Value::Number(item)) => range.push(item),
            (2, Value::Bytes(text)) => symbol = String::from_utf8(text.to_vec()).ok()?,
            (3, Value::Number(value)) => roles = value,
            _ => {}
        }
    }
    let [line, column, ..] = range[..] else {
        return None;
    };
    Some(Occurrence {
        line: usize::try_from(line).ok()?,
        column: usize::try_from(column).ok()?,
        symbol,
        is_definition: roles & DEFINITION_ROLE != 0,
    })
}
