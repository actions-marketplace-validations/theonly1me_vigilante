use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(test)]
mod tests;

const POLL: Duration = Duration::from_millis(100);

pub struct LimitedRun<'a> {
    pub command: &'a mut Command,
    pub input: Vec<u8>,
    pub deadline: Instant,
}

pub enum Finished {
    Exited(Output),
    OutOfTime,
}

pub fn run_limited(run: LimitedRun) -> std::io::Result<Finished> {
    let LimitedRun {
        command,
        input,
        deadline,
    } = run;
    let mut stdout = tempfile::tempfile()?;
    let mut stderr = tempfile::tempfile()?;
    let mut child = command
        .stdin(if input.is_empty() {
            Stdio::null()
        } else {
            Stdio::piped()
        })
        .stdout(Stdio::from(stdout.try_clone()?))
        .stderr(Stdio::from(stderr.try_clone()?))
        .spawn()?;
    let writer = child.stdin.take().map(|mut stdin| {
        thread::spawn(move || {
            stdin.write_all(&input).ok();
        })
    });
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break Some(status);
        }
        if Instant::now() >= deadline {
            child.kill().ok();
            child.wait()?;
            break None;
        }
        thread::sleep(POLL);
    };
    if let Some(writer) = writer {
        writer.join().ok();
    }
    let Some(status) = status else {
        return Ok(Finished::OutOfTime);
    };
    Ok(Finished::Exited(Output {
        status,
        stdout: read_all(&mut stdout)?,
        stderr: read_all(&mut stderr)?,
    }))
}

fn read_all(file: &mut File) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    file.seek(SeekFrom::Start(0))?;
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}
