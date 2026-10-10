use super::bytes::Reader;

const PE_SIGNATURE: u32 = 0x0000_4550;
const DOS_SIGNATURE: u16 = 0x5A4D;
const PE32: u16 = 0x10B;
const PE32_PLUS: u16 = 0x20B;
const CLI_DIRECTORY: usize = 14;
const SECTION_HEADER_LENGTH: usize = 40;
const METADATA_SIGNATURE: u32 = 0x424A_5342;
const DOS_HEADER_POINTER: usize = 0x3C;

struct Section {
    virtual_address: u32,
    virtual_size: u32,
    raw_offset: u32,
    raw_size: u32,
}

pub struct Image {
    sections: Vec<Section>,
}

pub struct Stream<'a> {
    pub name: &'a str,
    pub bytes: &'a [u8],
}

impl Image {
    pub fn read(bytes: &[u8]) -> Option<(Self, Vec<Stream<'_>>)> {
        let mut dos = Reader::new(bytes);
        if dos.u16()? != DOS_SIGNATURE {
            return None;
        }
        let header_start = Reader::new(bytes).at(DOS_HEADER_POINTER)?.u32()? as usize;
        let mut header = Reader::new(bytes).at(header_start)?;
        if header.u32()? != PE_SIGNATURE {
            return None;
        }
        header.skip(2)?;
        let section_count = usize::from(header.u16()?);
        header.skip(12)?;
        let optional_length = usize::from(header.u16()?);
        header.skip(2)?;
        let optional_start = header.position();
        let directories = match header.u16()? {
            PE32 => 96,
            PE32_PLUS => 112,
            _ => return None,
        };
        let mut directory =
            Reader::new(bytes).at(optional_start + directories + CLI_DIRECTORY * 8)?;
        let cli_address = directory.u32()?;
        let mut sections = Vec::with_capacity(section_count);
        let mut table = Reader::new(bytes).at(optional_start + optional_length)?;
        for _ in 0..section_count {
            let entry = table.take(SECTION_HEADER_LENGTH)?;
            let mut fields = Reader::new(entry).at(8)?;
            sections.push(Section {
                virtual_size: fields.u32()?,
                virtual_address: fields.u32()?,
                raw_size: fields.u32()?,
                raw_offset: fields.u32()?,
            });
        }
        let image = Self { sections };
        let mut cli = Reader::new(bytes).at(image.offset(cli_address)?)?;
        cli.skip(8)?;
        let metadata_address = cli.u32()?;
        let metadata_size = cli.u32()? as usize;
        let metadata_start = image.offset(metadata_address)?;
        let metadata = bytes.get(metadata_start..metadata_start.checked_add(metadata_size)?)?;
        let streams = streams(metadata)?;
        Some((image, streams))
    }

    pub fn offset(&self, address: u32) -> Option<usize> {
        self.sections.iter().find_map(|section| {
            let relative = address.checked_sub(section.virtual_address)?;
            (relative < section.virtual_size.max(section.raw_size))
                .then(|| (section.raw_offset + relative) as usize)
        })
    }
}

fn streams(metadata: &[u8]) -> Option<Vec<Stream<'_>>> {
    let mut root = Reader::new(metadata);
    if root.u32()? != METADATA_SIGNATURE {
        return None;
    }
    root.skip(8)?;
    let version_length = root.u32()? as usize;
    root.skip(version_length)?;
    root.skip(2)?;
    let count = root.u16()?;
    let mut found = Vec::new();
    for _ in 0..count {
        let offset = root.u32()? as usize;
        let size = root.u32()? as usize;
        let start = root.position();
        let name = root.text()?;
        let consumed = root.position() - start;
        root.skip((4 - consumed % 4) % 4)?;
        found.push(Stream {
            name,
            bytes: metadata.get(offset..offset.checked_add(size)?)?,
        });
    }
    Some(found)
}
