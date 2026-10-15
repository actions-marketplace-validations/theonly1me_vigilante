use std::io::ErrorKind;
use std::path::Path;
use std::process::{Command, Output};

use serde::Deserialize;

use crate::Error;

pub const MINIMUM_VERSION: (u64, u64, u64) = (0, 3, 3);
const INSTALL: &str = "go install github.com/google/capslock/cmd/capslock@v0.3.3";
const ARGUMENTS: &[&str] = &[
    "-packages=./...",
    "-output=json",
    "-granularity=intermediate",
];
const ERROR_LINES: usize = 20;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Report {
    #[serde(default)]
    capability_info: Vec<CapabilityInfo>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityInfo {
    pub capability_name: String,
    pub package_dir: String,
    #[serde(default)]
    pub path: Vec<Function>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Function {
    pub name: String,
    #[serde(default)]
    pub package: String,
    pub site: Option<Site>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Site {
    pub filename: String,
    pub line: String,
}

pub fn ensure_tools() -> Result<(), Error> {
    let missing_tool = |tool: &str| {
        Error::new(format!(
            "Vigilante needs `{tool}` to analyze Go modules with Capslock. Install Go and Capslock (`{INSTALL}`), or set `[engines] go = \"patterns\"` in vigilante.toml."
        ))
    };
    let capslock = run_tool(Command::new("capslock").arg("-version"))
        .map_err(|error| error.unwrap_or_else(|| missing_tool("capslock")))?;
    let text = String::from_utf8_lossy(&capslock.stdout);
    let version = text
        .split_whitespace()
        .find(|word| {
            word.strip_prefix('v')
                .is_some_and(|rest| rest.starts_with(|character: char| character.is_ascii_digit()))
        })
        .unwrap_or_default();
    match parse_version(version) {
        Some(found) if found >= MINIMUM_VERSION => {}
        _ => {
            return Err(Error::new(format!(
                "capslock {version} is older than v0.3.3 or has no version. Install it: `{INSTALL}`."
            )));
        }
    }
    run_tool(Command::new("go").arg("version"))
        .map_err(|error| error.unwrap_or_else(|| missing_tool("go")))?;
    Ok(())
}

pub fn analyze(module_folder: &Path) -> Result<Vec<CapabilityInfo>, Error> {
    let output = Command::new("capslock")
        .args(ARGUMENTS)
        .current_dir(module_folder)
        .env("GOFLAGS", "-mod=readonly")
        .output()
        .map_err(|error| Error::new(format!("cannot run capslock: {error}")))?;
    if !output.status.success() {
        let errors = String::from_utf8_lossy(&output.stderr);
        let lines: Vec<&str> = errors.lines().collect();
        let tail = lines[lines.len().saturating_sub(ERROR_LINES)..].join("\n");
        return Err(Error::new(format!(
            "capslock failed in {}:\n{tail}",
            module_folder.display()
        )));
    }
    let report: Report = serde_json::from_slice(&output.stdout)
        .map_err(|error| Error::new(format!("cannot read the capslock output: {error}")))?;
    Ok(report.capability_info)
}

fn run_tool(command: &mut Command) -> Result<Output, Option<Error>> {
    match command.output() {
        Ok(output) if output.status.success() => Ok(output),
        Ok(output) => Err(Some(Error::new(format!(
            "{command:?} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )))),
        Err(error) if error.kind() == ErrorKind::NotFound => Err(None),
        Err(error) => Err(Some(Error::new(format!("cannot run {command:?}: {error}")))),
    }
}

fn parse_version(text: &str) -> Option<(u64, u64, u64)> {
    let mut numbers = text.strip_prefix('v')?.split(['.', '-', '+']);
    Some((
        numbers.next()?.parse().ok()?,
        numbers.next()?.parse().ok()?,
        numbers.next()?.parse().ok()?,
    ))
}
