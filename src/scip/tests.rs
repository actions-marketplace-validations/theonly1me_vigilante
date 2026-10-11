use super::{Document, Occurrence, read_index};

fn varint(mut value: u64, out: &mut Vec<u8>) {
    while value >= 0x80 {
        out.push((value as u8) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

fn field(number: u64, bytes: &[u8], out: &mut Vec<u8>) {
    varint(number << 3 | 2, out);
    varint(bytes.len() as u64, out);
    out.extend_from_slice(bytes);
}

fn number(number: u64, value: u64, out: &mut Vec<u8>) {
    varint(number << 3, out);
    varint(value, out);
}

fn occurrence(range: &[u64], symbol: &str, roles: u64) -> Vec<u8> {
    let mut packed = Vec::new();
    for value in range {
        varint(*value, &mut packed);
    }
    let mut bytes = Vec::new();
    field(1, &packed, &mut bytes);
    field(2, symbol.as_bytes(), &mut bytes);
    number(3, roles, &mut bytes);
    bytes
}

fn index() -> Vec<u8> {
    let mut document = Vec::new();
    field(1, b"src/app.py", &mut document);
    field(
        2,
        &occurrence(&[1, 8, 11], "app/Logger#run().", 1),
        &mut document,
    );
    field(5, b"ignored text", &mut document);
    number(6, 2, &mut document);
    field(
        2,
        &occurrence(&[6, 18, 21], "app/Logger#run().", 8),
        &mut document,
    );
    let mut bytes = Vec::new();
    field(2, &document, &mut bytes);
    field(3, b"external symbols are skipped", &mut bytes);
    bytes
}

#[test]
fn documents_keep_their_definitions_and_references_in_order() {
    let index = read_index(&index()).expect("index");

    assert_eq!(
        index.documents,
        vec![Document {
            path: "src/app.py".to_string(),
            occurrences: vec![
                Occurrence {
                    line: 1,
                    column: 8,
                    symbol: "app/Logger#run().".to_string(),
                    is_definition: true,
                },
                Occurrence {
                    line: 6,
                    column: 18,
                    symbol: "app/Logger#run().".to_string(),
                    is_definition: false,
                },
            ],
        }]
    );
}

#[test]
fn a_cut_off_index_is_rejected() {
    let bytes = index();

    assert_eq!(read_index(&bytes[..bytes.len() - 3]), None);
}
