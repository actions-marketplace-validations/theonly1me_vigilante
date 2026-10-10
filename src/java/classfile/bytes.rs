pub struct Reader<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn is_done(&self) -> bool {
        self.position >= self.data.len()
    }

    pub fn u8(&mut self) -> Option<u8> {
        let value = *self.data.get(self.position)?;
        self.position += 1;
        Some(value)
    }

    pub fn u16(&mut self) -> Option<u16> {
        let bytes = self.bytes(2)?;
        Some(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    pub fn u32(&mut self) -> Option<u32> {
        let bytes = self.bytes(4)?;
        Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn i32(&mut self) -> Option<i32> {
        self.u32().map(|value| value as i32)
    }

    pub fn skip(&mut self, count: usize) -> Option<()> {
        self.bytes(count).map(|_| ())
    }

    pub fn bytes(&mut self, count: usize) -> Option<&'a [u8]> {
        let end = self.position.checked_add(count)?;
        let slice = self.data.get(self.position..end)?;
        self.position = end;
        Some(slice)
    }
}
