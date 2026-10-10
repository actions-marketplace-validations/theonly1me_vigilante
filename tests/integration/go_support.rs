use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

use serde_json::{Value, json};
use tempfile::TempDir;

use crate::support::{Run, TestRepository};

pub const SYSTEM_PATH: &str = "/usr/bin:/bin";
pub const APP_MODULE: &str = "app/go.mod";
pub const APP_MODULE_FILE: &str = "module example.com/app\n\ngo 1.22\n";
pub const FETCH_SOURCE: &str = "package fetch\n\nfunc Page(url string) error { return nil }\n";

const FAKE_GO: &str = "#!/bin/sh\necho \"go version go1.26.0 test/arch\"\n";
const FAKE_CAPSLOCK: &str = "#!/bin/sh
if [ \"$1\" = \"-version\" ]; then
  echo \"capslock version ${FAKE_CAPSLOCK_VERSION:-v0.3.3}\"
  exit 0
fi
echo \"$PWD $*\" >> \"$FAKE_CAPSLOCK_LOG\"
cat capslock.json
";

pub struct FakeTools {
    pub folder: TempDir,
}

pub enum Tools {
    Fake,
    Missing,
}

impl FakeTools {
    pub fn new() -> Self {
        let folder = TempDir::new().expect("temporary folder");
        for (name, script) in [("go", FAKE_GO), ("capslock", FAKE_CAPSLOCK)] {
            let path = folder.path().join(name);
            fs::write(&path, script).expect("fake tool");
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("executable");
        }
        Self { folder }
    }

    pub fn log(&self) -> String {
        fs::read_to_string(self.log_path()).unwrap_or_default()
    }

    fn log_path(&self) -> PathBuf {
        self.folder.path().join("capslock.log")
    }
}

pub struct GoRun<'a> {
    pub repository: &'a TestRepository,
    pub tools: &'a FakeTools,
    pub available: Tools,
    pub arguments: &'a [&'a str],
    pub environment: &'a [(&'a str, &'a str)],
}

pub fn run_vigilante(run: GoRun) -> Run {
    let search_path = match run.available {
        Tools::Fake => format!("{}:{SYSTEM_PATH}", run.tools.folder.path().display()),
        Tools::Missing => SYSTEM_PATH.to_string(),
    };
    let output = Command::new(env!("CARGO_BIN_EXE_vigilante"))
        .arg("--repository")
        .arg(run.repository.directory.path())
        .args(run.arguments)
        .env("PATH", search_path)
        .env("FAKE_CAPSLOCK_LOG", run.tools.log_path())
        .envs(run.environment.iter().copied())
        .output()
        .expect("vigilante runs");
    Run {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

pub struct Call<'a> {
    pub function: &'a str,
    pub package: &'a str,
    pub site: Option<(&'a str, u32)>,
}

pub struct Reach<'a> {
    pub package: &'a str,
    pub capability: &'a str,
    pub calls: &'a [Call<'a>],
}

pub fn capslock_output(reaches: &[Reach]) -> String {
    let infos: Vec<Value> = reaches
        .iter()
        .map(|reach| {
            let path: Vec<Value> = reach
                .calls
                .iter()
                .map(|call| {
                    let mut function = json!({ "name": call.function, "package": call.package });
                    if let Some((file, line)) = call.site {
                        function["site"] =
                            json!({ "filename": file, "line": line.to_string(), "column": "2" });
                    }
                    function
                })
                .collect();
            json!({
                "packageName": reach.package.rsplit('/').next(),
                "capabilityName": reach.capability,
                "capability": format!("CAPABILITY_{}", reach.capability.split('/').next().unwrap_or_default()),
                "path": path,
                "packageDir": reach.package,
            })
        })
        .collect();
    json!({ "capabilityInfo": infos, "moduleInfo": [], "packageInfo": [] }).to_string()
}

pub fn report_of(run: &Run) -> Value {
    serde_json::from_str(&run.stdout).unwrap_or_else(|_| panic!("JSON report: {}", run.stderr))
}
