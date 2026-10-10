use std::collections::BTreeMap;

use serde_json::Value;

use crate::csharp_fixtures::{Change, EXTRA, Feed, TOOLS, diff, project};
use crate::file_server::serve_files;
use crate::npm_registry::report_of;
use crate::python_fixtures::summaries;

const RUNNING_PROGRAM: &str = "using Acme.Tools;\n\nnamespace App;\n\npublic class Program\n{\n    public static void Main()\n    {\n        Shell.Run(1);\n    }\n}\n";
const EXEC_PATH: &str =
    "Program.Program.Main > Acme.Tools/Acme.Tools/Shell.Shell.Run > Process.Start";

#[test]
fn a_locked_package_that_runs_programs_is_a_note_with_its_call_path() {
    let feed = Feed::serve(&[&TOOLS, &EXTRA]);

    let run = diff(Change {
        base: &[
            ("App.csproj", &project(&[&TOOLS])),
            ("packages.lock.json", &feed.lockfile(&[&TOOLS, &EXTRA])),
            ("nuget.config", &feed.configuration()),
            ("vigilante.toml", &feed.allow_config()),
        ],
        head: &[("Program.cs", RUNNING_PROGRAM)],
    });

    let report = report_of(&run);
    assert_eq!(
        report["findings"],
        Value::Array(Vec::new()),
        "{}",
        run.stderr
    );
    let note = format!(". exec Acme.Tools Program.cs:9 {EXEC_PATH}");
    assert_eq!(
        summaries(&report["notes"]),
        vec![note.clone(), note],
        "{}",
        run.stderr
    );
}

#[test]
fn a_package_reference_without_a_lockfile_is_resolved_and_checked() {
    let feed = Feed::serve(&[&TOOLS, &EXTRA]);

    let run = diff(Change {
        base: &[
            ("App.csproj", &project(&[&TOOLS])),
            ("nuget.config", &feed.configuration()),
            ("vigilante.toml", &feed.allow_config()),
        ],
        head: &[("Program.cs", RUNNING_PROGRAM)],
    });

    let report = report_of(&run);
    let note = format!(". exec Acme.Tools Program.cs:9 {EXEC_PATH}");
    assert_eq!(
        summaries(&report["notes"]),
        vec![note.clone(), note],
        "{}",
        run.stderr
    );
    assert_eq!(
        report["skipped"],
        Value::Array(Vec::new()),
        "{}",
        run.stderr
    );
}

#[test]
fn a_package_that_does_not_match_its_lockfile_hash_stops_the_review() {
    let feed = Feed::serve(&[&TOOLS, &EXTRA]);
    let lockfile = feed.lockfile(&[&TOOLS, &EXTRA]);
    let hashes: Vec<&str> = lockfile
        .split("\"contentHash\": \"")
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
        .collect();
    let tampered = lockfile.replacen(hashes[0], hashes[1], 1);

    let run = diff(Change {
        base: &[
            ("App.csproj", &project(&[&TOOLS])),
            ("packages.lock.json", &tampered),
            ("nuget.config", &feed.configuration()),
            ("vigilante.toml", &feed.allow_config()),
        ],
        head: &[("Program.cs", RUNNING_PROGRAM)],
    });

    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(
        run.stderr.contains("does not match the checksum"),
        "{}",
        run.stderr
    );
}

#[test]
fn a_feed_that_is_not_allowed_is_listed() {
    let feed = Feed::serve(&[&TOOLS, &EXTRA]);

    let run = diff(Change {
        base: &[
            ("App.csproj", &project(&[&TOOLS])),
            ("nuget.config", &feed.configuration()),
        ],
        head: &[("Program.cs", RUNNING_PROGRAM)],
    });

    let reasons: Vec<String> = report_of(&run)["skipped"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|skip| skip["reason"].as_str().map(str::to_string))
        .collect();
    assert!(
        reasons.contains(&format!(
            "The registry `{}/v3/index.json` is not public. Add it to `[registries] allow` in vigilante.toml if it needs no credentials.",
            feed.base_url
        )),
        "{reasons:?}"
    );
}

#[test]
fn a_project_file_change_alone_selects_the_code_of_its_project() {
    let feed = Feed::serve(&[&TOOLS, &EXTRA]);

    let run = diff(Change {
        base: &[
            ("App.csproj", &project(&[])),
            ("Program.cs", RUNNING_PROGRAM),
            ("nuget.config", &feed.configuration()),
            ("vigilante.toml", &feed.allow_config()),
        ],
        head: &[("App.csproj", &project(&[&TOOLS]))],
    });

    let note = format!(". exec Acme.Tools Program.cs:9 {EXEC_PATH}");
    assert_eq!(
        summaries(&report_of(&run)["notes"]),
        vec![note.clone(), note],
        "{}",
        run.stderr
    );
}

#[test]
fn a_feed_that_is_not_allowed_is_not_read() {
    let base_url = serve_files(BTreeMap::from([(
        "/v3/index.json".to_string(),
        b"{\"resources\": [{\"@id\": \"https://api.nuget.org/v3-flatcontainer/\", \"@type\": \"PackageBaseAddress/3.0.0\"}]}".to_vec(),
    )]));

    let run = diff(Change {
        base: &[
            ("App.csproj", &project(&[])),
            (
                "nuget.config",
                &format!(
                    "<configuration><packageSources><clear /><add key=\"private\" value=\"{base_url}/v3/index.json\" /></packageSources></configuration>"
                ),
            ),
        ],
        head: &[("Program.cs", RUNNING_PROGRAM)],
    });

    let report = report_of(&run);
    let reasons: Vec<String> = report["skipped"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|skip| skip["reason"].as_str().map(str::to_string))
        .collect();
    assert_eq!(report["skipped"][0]["source"], "nuget.config");
    assert_eq!(
        reasons,
        vec![format!(
            "The registry `{base_url}/v3/index.json` is not public. Add it to `[registries] allow` in vigilante.toml if it needs no credentials."
        )]
    );
}
