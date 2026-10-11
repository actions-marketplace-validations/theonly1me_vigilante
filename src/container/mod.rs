use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::Error;

#[cfg(test)]
mod tests;

const DOCKER: &str = "docker";
const NETWORK: &str = "bridge";
const SOURCE: &str = "/source";
pub const OUTPUT: &str = "/output";
const POLL: Duration = Duration::from_millis(200);
const NO_DOCKER: &str = "Docker is not available";

pub struct ContainerRequest<'a> {
    pub image: &'a str,
    pub source: &'a Path,
    pub output: &'a Path,
    pub prepare: &'a str,
    pub build: &'a str,
    pub timeout: Duration,
}

pub enum ContainerRun {
    Built,
    NotBuilt(String),
}

pub fn run_contained(request: ContainerRequest) -> Result<ContainerRun, Error> {
    if !docker(&["version", "--format", "{{.Server.Version}}"])
        .is_ok_and(|output| output.status.success())
    {
        return Ok(ContainerRun::NotBuilt(NO_DOCKER.to_string()));
    }
    let deadline = Instant::now() + request.timeout;
    let source = format!("{}:{SOURCE}:ro", request.source.display());
    let output = format!("{}:{OUTPUT}", request.output.display());
    let seconds = request.timeout.as_secs().to_string();
    let started = docker(&[
        "run",
        "--detach",
        "--network",
        NETWORK,
        "--volume",
        &source,
        "--volume",
        &output,
        request.image,
        "sleep",
        &seconds,
    ])?;
    if !started.status.success() {
        return Ok(ContainerRun::NotBuilt(last_line(&started.stderr)));
    }
    let container = String::from_utf8_lossy(&started.stdout).trim().to_string();
    let run = steps(Steps {
        container: &container,
        request: &request,
        deadline,
    });
    docker(&["rm", "--force", &container]).ok();
    run
}

struct Steps<'a, 'r> {
    container: &'a str,
    request: &'a ContainerRequest<'r>,
    deadline: Instant,
}

fn steps(input: Steps) -> Result<ContainerRun, Error> {
    let Steps {
        container,
        request,
        deadline,
    } = input;
    let prepare = format!("cp -a {SOURCE} /work && cd /work && {}", request.prepare);
    let prepared = exec(Exec {
        arguments: &["exec", container, "sh", "-c", &prepare],
        deadline,
    })?;
    if !prepared.status.success() {
        return Ok(ContainerRun::NotBuilt(last_line(&prepared.stderr)));
    }
    let disconnected = docker(&["network", "disconnect", NETWORK, container])?;
    if !disconnected.status.success() {
        return Ok(ContainerRun::NotBuilt(last_line(&disconnected.stderr)));
    }
    let build = format!("cd /work && {}", request.build);
    let built = exec(Exec {
        arguments: &["exec", container, "sh", "-c", &build],
        deadline,
    })?;
    Ok(if built.status.success() {
        ContainerRun::Built
    } else {
        ContainerRun::NotBuilt(last_line(&built.stderr))
    })
}

fn docker(arguments: &[&str]) -> Result<Output, Error> {
    Command::new(DOCKER)
        .args(arguments)
        .output()
        .map_err(|error| Error::new(format!("{NO_DOCKER}: {error}")))
}

struct Exec<'a> {
    arguments: &'a [&'a str],
    deadline: Instant,
}

fn exec(input: Exec) -> Result<Output, Error> {
    let failed = |error: std::io::Error| Error::new(format!("docker did not finish: {error}"));
    let mut stdout = tempfile::tempfile().map_err(failed)?;
    let mut stderr = tempfile::tempfile().map_err(failed)?;
    let mut child = Command::new(DOCKER)
        .args(input.arguments)
        .stdout(Stdio::from(stdout.try_clone().map_err(failed)?))
        .stderr(Stdio::from(stderr.try_clone().map_err(failed)?))
        .spawn()
        .map_err(|error| Error::new(format!("{NO_DOCKER}: {error}")))?;
    let status = loop {
        if let Some(status) = child.try_wait().map_err(failed)? {
            break status;
        }
        if Instant::now() >= input.deadline {
            child.kill().ok();
            break child.wait().map_err(failed)?;
        }
        thread::sleep(POLL);
    };
    let read = |file: &mut std::fs::File| -> Result<Vec<u8>, Error> {
        let mut bytes = Vec::new();
        file.seek(SeekFrom::Start(0)).map_err(failed)?;
        file.read_to_end(&mut bytes).map_err(failed)?;
        Ok(bytes)
    };
    Ok(Output {
        status,
        stdout: read(&mut stdout)?,
        stderr: read(&mut stderr)?,
    })
}

fn last_line(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("no message")
        .trim()
        .to_string()
}
