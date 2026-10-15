use serde_json::Value;

use crate::python_fixtures::{Change, RUNNER, SPAWNER, diff, summaries};
use crate::python_index::PythonIndex;

#[test]
fn own_code_that_reaches_exec_through_a_relative_import_is_a_finding() {
    let report = diff(Change {
        base: &[],
        head: &[
            (
                "app.py",
                "from tools import build\n\n\ndef main():\n    return build.run_build()\n",
            ),
            ("tools/pyproject.toml", "[project]\nname = \"tools\"\n"),
            ("tools/__init__.py", ""),
            (
                "tools/build.py",
                "from .shell import call\n\n\ndef run_build():\n    return call('make')\n",
            ),
            (
                "tools/shell.py",
                "from subprocess import run\n\n\ndef call(command):\n    return run(command)\n",
            ),
        ],
    });

    let findings = summaries(&report["findings"]);

    assert_eq!(
        findings,
        vec![
            ". exec own app.py:5 app.main > tools/build.run_build > tools/shell.call > subprocess.run",
            "tools exec own tools/shell.py:5 tools/shell.call > subprocess.run",
        ]
    );
}

#[test]
fn only_the_module_that_a_pack_rule_names_is_privileged() {
    let report = diff(Change {
        base: &[],
        head: &[(
            "app.py",
            "import os, subprocess\n\n\ndef main():\n    return os.getcwd()\n",
        )],
    });

    assert_eq!(summaries(&report["findings"]), Vec::<String>::new());
}

#[test]
fn type_annotations_and_caught_exception_types_are_not_findings() {
    let report = diff(Change {
        base: &[],
        head: &[(
            "app.py",
            "import ctypes\nimport subprocess\nfrom typing import Optional\nfrom requests.exceptions import RequestException\n\n\nclass Guard:\n    process: Optional[subprocess.Popen] = None\n\n    def watch(self, current: subprocess.Popen) -> subprocess.Popen:\n        try:\n            return current.wait()\n        except (RequestException, ValueError) as error:\n            ctypes.CDLL(\"libc.so.6\")\n            raise error\n",
        )],
    });

    assert_eq!(
        summaries(&report["findings"]),
        vec![". native own app.py:14 app.Guard.watch > ctypes.CDLL"]
    );
}

#[test]
fn a_call_through_a_dotted_module_name_reaches_its_sink() {
    let report = diff(Change {
        base: &[],
        head: &[(
            "app.py",
            "import urllib.request\n\n\ndef fetch(url):\n    return urllib.request.urlopen(url)\n",
        )],
    });

    assert_eq!(
        summaries(&report["findings"]),
        vec![". network own app.py:5 app.fetch > urllib.request.urlopen"]
    );
}

#[test]
fn powers_that_do_not_block_are_notes_in_own_code() {
    let report = diff(Change {
        base: &[],
        head: &[(
            "app.py",
            "import os\n\n\ndef main(code):\n    token = os.environ['TOKEN']\n    return eval(code)\n",
        )],
    });

    assert_eq!(summaries(&report["findings"]), Vec::<String>::new());
    assert_eq!(
        summaries(&report["notes"]),
        vec![
            ". dynamic_code . app.py:6 app.main > eval",
            ". read_system_state . app.py:5 app.main > os.environ",
        ]
    );
}

#[test]
fn a_new_package_that_runs_programs_is_a_note_with_its_call_path() {
    let index = PythonIndex::serve(&[&RUNNER, &SPAWNER]);
    let lock = index.uv_lock(&[&RUNNER, &SPAWNER]);

    let report = diff(Change {
        base: &[
            ("vigilante.toml", &index.allow_config()),
            ("uv.lock", &lock),
        ],
        head: &[(
            "app.py",
            "from runner import begin\n\n\ndef main():\n    return begin(['ls'])\n",
        )],
    });

    let path =
        "app.main > runner/runner/core.start > spawner/spawner/__init__.spawn_it > subprocess.run";
    assert_eq!(report["findings"], Value::Array(Vec::new()));
    assert_eq!(
        summaries(&report["notes"]),
        vec![
            format!(". exec runner app.py:5 {path}"),
            format!(". exec runner app.py:5 {path}"),
            format!(". exec spawner app.py:5 {path}"),
        ]
    );
}

#[test]
fn an_import_that_is_neither_in_the_repository_nor_in_the_lockfile_is_listed() {
    let index = PythonIndex::serve(&[&RUNNER, &SPAWNER]);
    let lock = index.uv_lock(&[&RUNNER, &SPAWNER]);

    let report = diff(Change {
        base: &[
            ("vigilante.toml", &index.allow_config()),
            ("uv.lock", &lock),
        ],
        head: &[(
            "app.py",
            "import ghost\n\n\ndef main():\n    return ghost.run()\n",
        )],
    });

    assert_eq!(
        report["skipped"][0]["reason"],
        "Vigilante found `ghost` neither in the repository nor in the lockfile, so it did not read it."
    );
}

#[test]
fn importing_a_module_runs_its_top_level_code() {
    let report = diff(Change {
        base: &[],
        head: &[
            (
                "app.py",
                "import tools.boot\n\n\ndef main():\n    return 1\n",
            ),
            ("tools/pyproject.toml", "[project]\nname = \"tools\"\n"),
            ("tools/__init__.py", ""),
            (
                "tools/boot.py",
                "import subprocess\n\nsubprocess.run(['make'])\n",
            ),
        ],
    });

    assert_eq!(
        summaries(&report["findings"]),
        vec![
            ". exec own app.py:1 app.init > tools/boot.init > subprocess.run",
            "tools exec own tools/boot.py:3 tools/boot.init > subprocess.run",
        ]
    );
}
