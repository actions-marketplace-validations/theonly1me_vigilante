use super::bytes::Reader;
use super::schema::{Column, TABLE_COUNT, columns, tag_bits};

const LARGE_STRINGS: u8 = 0x01;
const LARGE_GUIDS: u8 = 0x02;
const LARGE_BLOBS: u8 = 0x04;
const EXTRA_DATA: u8 = 0x40;
const SMALL_LIMIT: usize = 1 << 16;

pub struct Tables<'a> {
    bytes: &'a [u8],
    rows: [usize; TABLE_COUNT],
    starts: [usize; TABLE_COUNT],
    row_sizes: [usize; TABLE_COUNT],
    widths: Vec<Vec<usize>>,
    strings: &'a [u8],
    blobs: &'a [u8],
}

pub struct Heaps<'a> {
    pub tables: &'a [u8],
    pub strings: &'a [u8],
    pub blobs: &'a [u8],
}

pub struct Cell {
    pub table: usize,
    pub row: usize,
    pub column: usize,
}

impl<'a> Tables<'a> {
    pub fn read(heaps: Heaps<'a>) -> Option<Self> {
        let mut reader = Reader::new(heaps.tables);
        reader.skip(6)?;
        let heap_sizes = reader.u8()?;
        reader.skip(1)?;
        let valid = u64::from(reader.u32()?) | (u64::from(reader.u32()?) << 32);
        reader.skip(8)?;
        let mut rows = [0; TABLE_COUNT];
        for (table, count) in rows.iter_mut().enumerate() {
            if valid & (1 << table) != 0 {
                *count = reader.u32()? as usize;
            }
        }
        if heap_sizes & EXTRA_DATA != 0 {
            reader.skip(4)?;
        }
        let heap_width = |flag: u8| if heap_sizes & flag != 0 { 4 } else { 2 };
        let index_width = |table: usize| if rows[table] < SMALL_LIMIT { 2 } else { 4 };
        let coded_width = |tables: &[usize]| {
            let limit = 1usize << (16 - tag_bits(tables));
            let largest = tables
                .iter()
                .map(|table| rows.get(*table).copied().unwrap_or(0))
                .max()
                .unwrap_or(0);
            if largest < limit { 2 } else { 4 }
        };
        let widths: Vec<Vec<usize>> = (0..TABLE_COUNT)
            .map(|table| {
                columns(table)
                    .iter()
                    .map(|column| match column {
                        Column::Fixed(width) => *width,
                        Column::Strings => heap_width(LARGE_STRINGS),
                        Column::Guid => heap_width(LARGE_GUIDS),
                        Column::Blob => heap_width(LARGE_BLOBS),
                        Column::Table(target) => index_width(*target),
                        Column::Coded(targets) => coded_width(targets),
                    })
                    .collect()
            })
            .collect();
        let mut starts = [0; TABLE_COUNT];
        let mut row_sizes = [0; TABLE_COUNT];
        let mut position = reader.position();
        for table in 0..TABLE_COUNT {
            if rows[table] > 0 && columns(table).is_empty() {
                return None;
            }
            row_sizes[table] = widths[table].iter().sum();
            starts[table] = position;
            position = position.checked_add(rows[table].checked_mul(row_sizes[table])?)?;
        }
        (position <= heaps.tables.len()).then_some(Self {
            bytes: heaps.tables,
            rows,
            starts,
            row_sizes,
            widths,
            strings: heaps.strings,
            blobs: heaps.blobs,
        })
    }

    pub fn rows(&self, table: usize) -> usize {
        self.rows[table]
    }

    pub fn value(&self, cell: Cell) -> Option<u32> {
        let Cell { table, row, column } = cell;
        if row == 0 || row > self.rows[table] {
            return None;
        }
        let offset: usize = self.widths[table][..column].iter().sum();
        let position = self.starts[table] + (row - 1) * self.row_sizes[table] + offset;
        let width = self.widths[table][column];
        let mut reader = Reader::new(self.bytes).at(position)?;
        match width {
            1 => reader.u8().map(u32::from),
            2 => reader.u16().map(u32::from),
            4 => reader.u32(),
            _ => None,
        }
    }

    pub fn text(&self, cell: Cell) -> Option<&'a str> {
        let index = self.value(cell)? as usize;
        Reader::new(self.strings).at(index)?.text()
    }

    pub fn blob(&self, cell: Cell) -> Option<&'a [u8]> {
        let index = self.value(cell)? as usize;
        let mut reader = Reader::new(self.blobs).at(index)?;
        let length = reader.compressed()? as usize;
        reader.take(length)
    }
}
