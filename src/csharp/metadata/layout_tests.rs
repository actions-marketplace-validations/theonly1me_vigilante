use super::il::invocations;
use super::schema::{MODULE, TYPE_DEFINITION, TYPE_REFERENCE};
use super::tables::{Cell, Heaps, Tables};

const CALL_AFTER_SWITCH: &[u8] = &[
    0x45, 0x01, 0x00, 0x00, 0x00, 0x28, 0x05, 0x00, 0x00, 0x28, 0x01, 0x00, 0x00, 0x0A, 0x2A,
];
const WIDE_CONSTANTS: &[u8] = &[
    0x21, 0x28, 0x00, 0x00, 0x00, 0x28, 0x00, 0x00, 0x00, 0x23, 0x28, 0x00, 0x00, 0x00, 0x28, 0x00,
    0x00, 0x00, 0x2A,
];
const LARGE_ROW_COUNT: u32 = 1 << 14;
const NAME_MARK: u16 = 0x1234;

#[test]
fn a_switch_skips_its_jump_targets() {
    let found: Vec<u32> = invocations(CALL_AFTER_SWITCH)
        .expect("code")
        .iter()
        .map(|invocation| invocation.token)
        .collect();

    assert_eq!(found, vec![0x0A00_0001]);
}

#[test]
fn eight_byte_constants_are_skipped_whole() {
    assert!(invocations(WIDE_CONSTANTS).expect("code").is_empty());
}

#[test]
fn a_coded_index_grows_to_four_bytes_when_a_target_table_is_large() {
    let mut stream = vec![0, 0, 0, 0, 2, 0, 0, 0];
    let valid: u64 = (1 << MODULE) | (1 << TYPE_REFERENCE) | (1 << TYPE_DEFINITION);
    stream.extend(valid.to_le_bytes());
    stream.extend(0u64.to_le_bytes());
    for rows in [1, LARGE_ROW_COUNT, 1] {
        stream.extend(rows.to_le_bytes());
    }
    stream.extend([0; 10]);
    stream.extend(vec![0; LARGE_ROW_COUNT as usize * 8]);
    stream.extend(0u32.to_le_bytes());
    stream.extend(NAME_MARK.to_le_bytes());
    stream.extend([0; 10]);

    let tables = Tables::read(Heaps {
        tables: &stream,
        strings: &[],
        blobs: &[],
    })
    .expect("tables");

    assert_eq!(
        tables.value(Cell {
            table: TYPE_DEFINITION,
            row: 1,
            column: 1,
        }),
        Some(u32::from(NAME_MARK))
    );
}
