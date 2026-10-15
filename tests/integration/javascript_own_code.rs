use serde_json::Value;

use crate::npm_registry::report_of;
use crate::support::TestRepository;

const DIFF: &[&str] = &[
    "diff", "--base", "main", "--head", "feature", "--format", "json",
];
const MANIFEST: (&str, &str) = ("package.json", "{\"name\": \"app\"}");

pub fn diff_after(change: &[(&str, &str)]) -> Value {
    diff_with_base(Files { base: &[], change })
}

struct Files<'a> {
    base: &'a [(&'a str, &'a str)],
    change: &'a [(&'a str, &'a str)],
}

fn diff_with_base(files: Files) -> Value {
    let Files { base, change } = files;
    let repository = TestRepository::new();
    repository.write(&[MANIFEST, ("src/index.js", "export function main() {}\n")]);
    repository.write(base);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(change);
    repository.commit("change");
    report_of(&repository.run_with_cache(DIFF))
}

pub fn entries(list: &Value) -> Vec<String> {
    list.as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|entry| {
            let call_path: Vec<&str> = entry["evidence"]["call_path"]
                .as_array()
                .map(|names| names.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            format!(
                "{} {} {}:{} {}",
                entry["scope"]["path"].as_str().unwrap_or_default(),
                entry["power"].as_str().unwrap_or_default(),
                entry["evidence"]["path"].as_str().unwrap_or_default(),
                entry["evidence"]["line"],
                call_path.join(" > ")
            )
        })
        .collect()
}

#[test]
fn own_code_that_reaches_exec_through_another_package_folder_is_a_finding() {
    let report = diff_after(&[
        (
            "src/index.js",
            "import { build } from '../tools/build.js';\n\nexport function main() {\n  return build();\n}\n",
        ),
        ("tools/package.json", "{\"name\": \"tools\"}"),
        (
            "tools/build.js",
            "import { execSync } from 'node:child_process';\n\nexport function build() {\n  return compile();\n}\n\nfunction compile() {\n  return execSync('tsc');\n}\n",
        ),
    ]);

    let findings = entries(&report["findings"]);

    assert!(
        findings.contains(
            &". exec src/index.js:4 src/index.main > tools/build.build > tools/build.compile > child_process.execSync"
                .to_string()
        ),
        "{findings:#?}"
    );
    assert!(
        findings.contains(
            &"tools exec tools/build.js:8 tools/build.compile > child_process.execSync".to_string()
        ),
        "{findings:#?}"
    );
}

#[test]
fn an_import_that_is_never_called_is_not_a_power() {
    let report = diff_after(&[(
        "src/index.js",
        "import { exec } from 'child_process';\nconst cp = require('child_process');\n\nexport function main() {\n  return 1;\n}\n",
    )]);

    assert_eq!(entries(&report["findings"]), Vec::<String>::new());
}

#[test]
fn common_js_calls_through_a_required_module_are_followed() {
    let report = diff_after(&[(
        "src/index.js",
        "const cp = require('child_process');\n\nmodule.exports = function run(command) {\n  return cp.spawn(command);\n};\n",
    )]);

    assert_eq!(
        entries(&report["findings"]),
        vec![". exec src/index.js:4 src/index.run > child_process.spawn"]
    );
}

#[test]
fn a_package_in_a_pack_rule_is_a_sink_in_own_code() {
    let report = diff_after(&[(
        "src/index.ts",
        "import axios from 'axios';\n\nexport async function load(url: string) {\n  return axios.get(url);\n}\n",
    )]);

    assert_eq!(
        entries(&report["findings"]),
        vec![". network src/index.ts:4 src/index.load > axios.get"]
    );
    assert_eq!(report["skipped"], Value::Array(Vec::new()));
}

#[test]
fn powers_that_do_not_block_are_notes_in_own_code() {
    let report = diff_after(&[(
        "src/index.js",
        "export function main(code) {\n  const token = process.env.TOKEN;\n  return eval(code);\n}\n",
    )]);

    assert_eq!(entries(&report["findings"]), Vec::<String>::new());
    assert_eq!(
        entries(&report["notes"]),
        vec![
            ". dynamic_code src/index.js:3 src/index.main > eval",
            ". read_system_state src/index.js:2 src/index.main > process.env",
        ]
    );
}

#[test]
fn the_patterns_engine_reports_an_import_without_the_call_graph() {
    let report = diff_with_base(Files {
        base: &[("vigilante.toml", "[engines]\njavascript = \"patterns\"\n")],
        change: &[("src/index.js", "import { exec } from 'child_process';\n")],
    });

    assert_eq!(entries(&report["findings"]), vec![". exec src/index.js:1 "]);
}

#[test]
fn a_relative_module_whose_name_ends_in_node_is_code_when_the_file_exists() {
    let report = diff_after(&[
        (
            "src/index.ts",
            "export async function register() {\n  await import('./instrumentation.node');\n  return require('./addon.node');\n}\n",
        ),
        (
            "src/instrumentation.node.ts",
            "export const ready = true;\n",
        ),
    ]);

    assert_eq!(
        entries(&report["findings"]),
        vec![". native src/index.ts:3 src/index.register > ./addon.node"]
    );
}
