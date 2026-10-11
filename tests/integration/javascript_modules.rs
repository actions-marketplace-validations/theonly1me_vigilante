use crate::javascript_own_code::{diff_after, entries};

#[test]
fn importing_a_module_runs_its_top_level_code() {
    let report = diff_after(&[
        (
            "src/index.js",
            "import '../tools/boot.js';\n\nexport function main() {}\n",
        ),
        ("tools/package.json", "{\"name\": \"tools\"}"),
        (
            "tools/boot.js",
            "import { execSync } from 'child_process';\n\nexecSync('make');\n",
        ),
    ]);

    assert_eq!(
        entries(&report["findings"]),
        vec!["tools exec tools/boot.js:3 tools/boot.init > child_process.execSync"]
    );
    assert_eq!(
        entries(&report["notes"]),
        vec![". exec src/index.js:1 src/index.init > tools/boot.init > child_process.execSync"]
    );
}

#[test]
fn a_call_by_method_name_does_not_reach_the_top_level_code_of_other_files() {
    let report = diff_after(&[
        (
            "src/index.js",
            "import { helper } from '../tools/helper.js';\n\nexport function main(thing) {\n  helper();\n  return thing.init();\n}\n",
        ),
        ("tools/package.json", "{\"name\": \"tools\"}"),
        ("tools/helper.js", "export function helper() {}\n"),
        (
            "tools/boot.js",
            "import { execSync } from 'child_process';\n\nexecSync('make');\n",
        ),
    ]);

    assert_eq!(
        entries(&report["findings"]),
        vec!["tools exec tools/boot.js:3 tools/boot.init > child_process.execSync"]
    );
}
