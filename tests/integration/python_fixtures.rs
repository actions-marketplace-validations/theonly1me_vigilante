use serde_json::Value;

use crate::npm_registry::report_of;
use crate::python_index::FakeWheel;
use crate::support::TestRepository;

const DIFF: &[&str] = &[
    "diff", "--base", "main", "--head", "feature", "--format", "json",
];
const PROJECT: (&str, &str) = (
    "pyproject.toml",
    "[project]\nname = \"app\"\nversion = \"0.1.0\"\n",
);
pub const RUNNER: FakeWheel = FakeWheel {
    name: "runner",
    version: "1.0.0",
    files: &[
        ("runner/__init__.py", "from .core import start as begin\n"),
        (
            "runner/core.py",
            "from spawner import spawn_it\n\n\ndef start(command):\n    return spawn_it(command)\n",
        ),
    ],
};
pub const SPAWNER: FakeWheel = FakeWheel {
    name: "spawner",
    version: "2.0.0",
    files: &[(
        "spawner/__init__.py",
        "import subprocess\n\n\ndef spawn_it(command):\n    return subprocess.run(command)\n",
    )],
};

pub struct Change<'a> {
    pub base: &'a [(&'a str, &'a str)],
    pub head: &'a [(&'a str, &'a str)],
}

pub fn diff(change: Change) -> Value {
    let repository = TestRepository::new();
    repository.write(&[PROJECT, ("app.py", "def main():\n    return 0\n")]);
    repository.write(change.base);
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    repository.write(change.head);
    repository.commit("change");
    report_of(&repository.run_with_cache(DIFF))
}

pub fn summaries(list: &Value) -> Vec<String> {
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
                "{} {} {} {}:{} {}",
                entry["scope"]["path"].as_str().unwrap_or_default(),
                entry["power"].as_str().unwrap_or_default(),
                entry["through"].as_str().unwrap_or("own"),
                entry["evidence"]["path"].as_str().unwrap_or_default(),
                entry["evidence"]["line"],
                call_path.join(" > ")
            )
        })
        .collect()
}
