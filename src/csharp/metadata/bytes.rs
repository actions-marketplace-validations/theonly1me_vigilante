pub struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    pub fn at(self, position: usize) -> Option<Self> {
        (position <= self.bytes.len()).then_some(Self {
            bytes: self.bytes,
            position,
        })
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn is_done(&self) -> bool {
        self.position >= self.bytes.len()
    }

    pub fn skip(&mut self, count: usize) -> Option<()> {
        let end = self.position.checked_add(count)?;
        (end <= self.bytes.len()).then(|| self.position = end)
    }

    pub fn take(&mut self, count: usize) -> Option<&'a [u8]> {
        let start = self.position;
        self.skip(count)?;
        Some(&self.bytes[start..self.position])
    }

    pub fn u8(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }

    pub fn u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.take(2)?.try_into().ok()?))
    }

    pub fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    pub fn compressed(&mut self) -> Option<u32> {
        let first = u32::from(self.u8()?);
        if first & 0x80 == 0 {
            return Some(first);
        }
        if first & 0xC0 == 0x80 {
            return Some(((first & 0x3F) << 8) | u32::from(self.u8()?));
        }
        let rest = self.take(3)?;
        Some(
            ((first & 0x1F) << 24)
                | (u32::from(rest[0]) << 16)
                | (u32::from(rest[1]) << 8)
                | u32::from(rest[2]),
        )
    }

    pub fn text(&mut self) -> Option<&'a str> {
        let rest = &self.bytes[self.position..];
        let length = rest.iter().position(|byte| *byte == 0)?;
        let text = std::str::from_utf8(&rest[..length]).ok()?;
        self.position += length + 1;
        Some(text)
    }
}
