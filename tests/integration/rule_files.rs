use crate::support::{expected, scan_files, sightings};

const UNSAFE_IMPORT: &str = "package api\n\nimport \"unsafe\"\n";

const INTERNAL_CLIENT_RULE: &str = "id: internal-http-client
language: python
metadata:
  capability: network
files:
  - services/**
rule:
  pattern: internal_http.get($$$)
";

#[test]
fn generated_protobuf_go_code_does_not_report_native() {
    let map = scan_files(&[
        ("go.mod", "module example.com/app\n"),
        ("api/client.pb.go", UNSAFE_IMPORT),
        ("api/pool.go", UNSAFE_IMPORT),
    ]);

    assert_eq!(sightings(&map), expected(&["code . native api/pool.go:3"]));
}

#[test]
fn a_custom_rule_runs_only_on_its_files() {
    let call = "internal_http.get(\"https://billing.internal\")\n";
    let map = scan_files(&[
        (
            "vigilante.toml",
            "[rules]\npaths = [\".vigilante/rules\"]\n",
        ),
        (".vigilante/rules/internal.yml", INTERNAL_CLIENT_RULE),
        ("services/billing/export.py", call),
        ("tools/report.py", call),
    ]);

    assert_eq!(
        sightings(&map),
        expected(&["code services network services/billing/export.py:1"])
    );
}
