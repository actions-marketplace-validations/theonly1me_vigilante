use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::support::scan_files;

const ASSIGNED_NAMES: usize = 40_000;
const FUNCTIONS: usize = 10_000;
const TIME_LIMIT: Duration = Duration::from_secs(20);

fn bundle_with_many_assignments_and_calls() -> String {
    let assignments = (0..ASSIGNED_NAMES).map(|number| format!("var name{number} = {number};\n"));
    let calls = (0..ASSIGNED_NAMES).map(|number| format!("log(name{number});\n"));
    assignments.chain(calls).collect()
}

fn bundle_with_many_functions() -> String {
    (0..FUNCTIONS)
        .map(|number| format!("function run{number}(value) {{ log(value, run{number}); }}\n"))
        .collect()
}

fn scan_finishes_in_time(source: String) -> bool {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let map = scan_files(&[
            ("package.json", "{\"name\": \"app\"}"),
            ("vigilante.toml", "[engines]\njavascript = \"deep\"\n"),
            ("bundle.js", &source),
        ]);
        sender.send(map.scopes.len()).expect("result");
    });
    receiver.recv_timeout(TIME_LIMIT).is_ok()
}

#[test]
fn a_file_with_thousands_of_assignments_and_calls_is_read_in_a_short_time() {
    assert!(
        scan_finishes_in_time(bundle_with_many_assignments_and_calls()),
        "the scan did not finish in {TIME_LIMIT:?}"
    );
}

#[test]
fn a_file_with_thousands_of_functions_is_read_in_a_short_time() {
    assert!(
        scan_finishes_in_time(bundle_with_many_functions()),
        "the scan did not finish in {TIME_LIMIT:?}"
    );
}
