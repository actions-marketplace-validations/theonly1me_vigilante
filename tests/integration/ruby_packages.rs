use serde_json::Value;

use crate::npm_registry::report_of;
use crate::python_fixtures::summaries;
use crate::ruby_fixtures::{Change, GemServer, RUNNER, SPAWNER, diff};

const RUNNING_APP: &str =
    "require \"runner\"\n\nclass App\n  def main\n    Runner.start\n  end\nend\n";
const PATH: &str =
    "app.App.main > runner/lib/runner.Runner.start > spawner/lib/spawner.Spawner.spawn_it > system";

fn reasons(run: &crate::support::Run) -> Vec<String> {
    report_of(run)["skipped"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|skip| skip["reason"].as_str().map(str::to_string))
        .collect()
}

#[test]
fn a_new_gem_that_runs_programs_is_a_note_with_its_call_path() {
    let server = GemServer::serve(&[&RUNNER, &SPAWNER]);

    let run = diff(Change {
        base: &[
            (
                "Gemfile",
                "source \"https://rubygems.org\"\ngem \"runner\"\n",
            ),
            ("Gemfile.lock", &server.lockfile(&[&RUNNER, &SPAWNER])),
            ("vigilante.toml", &server.allow_config()),
        ],
        head: &[("app.rb", RUNNING_APP)],
    });

    let report = report_of(&run);
    assert_eq!(
        report["findings"],
        Value::Array(Vec::new()),
        "{}",
        run.stderr
    );
    assert_eq!(
        summaries(&report["notes"]),
        vec![
            format!(". exec runner app.rb:5 {PATH}"),
            format!(". exec runner app.rb:5 {PATH}"),
            format!(". exec spawner app.rb:5 {PATH}"),
        ],
        "{}",
        run.stderr
    );
}

#[test]
fn a_gem_that_does_not_match_its_lockfile_checksum_stops_the_review() {
    let server = GemServer::serve(&[&RUNNER, &SPAWNER]);
    let checksums = server.checksums(&[&RUNNER, &SPAWNER]);
    let values: Vec<&str> = checksums
        .lines()
        .filter_map(|line| line.split("sha256=").nth(1))
        .collect();
    let tampered = checksums.replacen(values[0], values[1], 1);

    let run = diff(Change {
        base: &[
            (
                "Gemfile",
                "source \"https://rubygems.org\"\ngem \"runner\"\n",
            ),
            (
                "Gemfile.lock",
                &format!("{}{tampered}", server.lockfile(&[&RUNNER, &SPAWNER])),
            ),
            ("vigilante.toml", &server.allow_config()),
        ],
        head: &[("app.rb", RUNNING_APP)],
    });

    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(
        run.stderr.contains("does not match the checksum"),
        "{}",
        run.stderr
    );
}

#[test]
fn a_registry_that_is_not_allowed_is_listed() {
    let server = GemServer::serve(&[&RUNNER, &SPAWNER]);

    let run = diff(Change {
        base: &[
            (
                "Gemfile",
                "source \"https://rubygems.org\"\ngem \"runner\"\n",
            ),
            ("Gemfile.lock", &server.lockfile(&[&RUNNER, &SPAWNER])),
        ],
        head: &[("app.rb", RUNNING_APP)],
    });

    assert_eq!(
        reasons(&run),
        vec![format!(
            "The registry `{}/` is not public. Add it to `[registries] allow` in vigilante.toml if it needs no credentials.",
            server.base_url
        )]
    );
}

#[test]
fn a_required_gem_that_is_not_locked_or_code_without_a_lockfile_is_listed() {
    let server = GemServer::serve(&[&SPAWNER]);

    let missing = diff(Change {
        base: &[
            ("Gemfile", "source \"https://rubygems.org\"\n"),
            ("Gemfile.lock", &server.lockfile(&[&SPAWNER])),
            ("vigilante.toml", &server.allow_config()),
        ],
        head: &[("app.rb", RUNNING_APP)],
    });
    let unlocked = diff(Change {
        base: &[("Gemfile", "source \"https://rubygems.org\"\n")],
        head: &[("app.rb", RUNNING_APP)],
    });

    assert_eq!(
        reasons(&missing),
        vec![
            "Vigilante found `runner` neither in the repository nor in Gemfile.lock, so it did not read it."
        ]
    );
    assert_eq!(
        reasons(&unlocked),
        vec!["No Gemfile.lock covers this code, so Vigilante did not read its gems."]
    );
}

#[test]
fn a_required_file_is_found_in_a_gem_with_another_name() {
    let rubyzip = crate::ruby_fixtures::FakeGem {
        name: "rubyzip",
        version: "2.3.2",
        dependencies: &[],
        files: &[(
            "lib/zip.rb",
            "module Zip\n  def self.extract\n    system(\"unzip\")\n  end\nend\n",
        )],
    };
    let server = GemServer::serve(&[&rubyzip]);

    let run = diff(Change {
        base: &[
            (
                "Gemfile",
                "source \"https://rubygems.org\"\ngem \"rubyzip\"\n",
            ),
            ("Gemfile.lock", &server.lockfile(&[&rubyzip])),
            ("vigilante.toml", &server.allow_config()),
        ],
        head: &[(
            "app.rb",
            "require \"zip\"\n\nclass App\n  def main\n    Zip.extract\n  end\nend\n",
        )],
    });

    let report = report_of(&run);
    let note =
        ". exec rubyzip app.rb:5 app.App.main > rubyzip/lib/zip.Zip.extract > system".to_string();
    assert_eq!(
        summaries(&report["notes"]),
        vec![note.clone(), note],
        "{}",
        run.stderr
    );
    assert_eq!(report["skipped"], Value::Array(Vec::new()));
}
