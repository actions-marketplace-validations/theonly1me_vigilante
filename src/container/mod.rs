use std::path::Path;
use std::process::{Command, Output};
use std::time::{Duration, Instant};

use crate::Error;
use crate::process::{Finished, LimitedRun, run_limited};

#[cfg(test)]
mod tests;

const DOCKER: &str = "docker";
const NETWORK: &str = "bridge";
const SOURCE: &str = "/source";
pub const OUTPUT: &str = "/output";
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
    OutOfTime,
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
    let Some(prepared) = exec(Exec {
        arguments: &["exec", container, "sh", "-c", &prepare],
        deadline,
    })?
    else {
        return Ok(ContainerRun::OutOfTime);
    };
    if !prepared.status.success() {
        return Ok(ContainerRun::NotBuilt(last_line(&prepared.stderr)));
    }
    let disconnected = docker(&["network", "disconnect", NETWORK, container])?;
    if !disconnected.status.success() {
        return Ok(ContainerRun::NotBuilt(last_line(&disconnected.stderr)));
    }
    let build = format!("cd /work && {}", request.build);
    let Some(built) = exec(Exec {
        arguments: &["exec", container, "sh", "-c", &build],
        deadline,
    })?
    else {
        return Ok(ContainerRun::OutOfTime);
    };
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

fn exec(input: Exec) -> Result<Option<Output>, Error> {
    let finished = run_limited(LimitedRun {
        command: Command::new(DOCKER).args(input.arguments),
        input: Vec::new(),
        deadline: input.deadline,
    })
    .map_err(|error| Error::new(format!("{NO_DOCKER}: {error}")))?;
    Ok(match finished {
        Finished::Exited(output) => Some(output),
        Finished::OutOfTime => None,
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
