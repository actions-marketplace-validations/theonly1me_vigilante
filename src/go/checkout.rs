use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

use crate::Error;

pub struct Checkout {
    folder: TempDir,
}

pub struct CheckoutRequest<'a> {
    pub repository: &'a Path,
    pub commit: &'a str,
}

impl Checkout {
    pub fn new(request: CheckoutRequest) -> Result<Self, Error> {
        let folder = tempfile::Builder::new()
            .prefix("vigilante-checkout-")
            .tempdir()
            .map_err(|error| Error::new(format!("cannot create a temporary folder: {error}")))?;
        let source = request.repository.to_string_lossy();
        let target = folder.path().to_string_lossy();
        git(&[
            "clone",
            "--shared",
            "--no-checkout",
            "--quiet",
            "--",
            &source,
            &target,
        ])?;
        git(&[
            "-C",
            &target,
            "checkout",
            "--quiet",
            "--detach",
            request.commit,
        ])?;
        Ok(Self { folder })
    }

    pub fn root(&self) -> &Path {
        self.folder.path()
    }
}

fn git(arguments: &[&str]) -> Result<(), Error> {
    let output = Command::new("git")
        .args([
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.fsmonitor=false",
        ])
        .args(arguments)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_LFS_SKIP_SMUDGE", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|error| Error::new(format!("cannot run git: {error}")))?;
    if output.status.success() {
        return Ok(());
    }
    Err(Error::new(format!(
        "git {} failed: {}",
        arguments.join(" "),
        String::from_utf8_lossy(&output.stderr).trim()
    )))
}
