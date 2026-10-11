use std::fs;
use std::process::Command;
use std::time::{Duration, Instant};

use super::{Finished, LimitedRun, run_limited};

#[test]
fn a_command_that_finishes_in_time_gets_its_input_and_returns_its_output() {
    let finished = run_limited(LimitedRun {
        command: &mut Command::new("cat"),
        input: b"calls".to_vec(),
        deadline: Instant::now() + Duration::from_secs(10),
    })
    .expect("cat runs");

    let Finished::Exited(output) = finished else {
        panic!("cat reached the time limit");
    };
    assert!(output.status.success());
    assert_eq!(output.stdout, b"calls");
}

#[test]
fn a_command_that_reaches_the_deadline_is_stopped() {
    let folder = tempfile::tempdir().expect("folder");
    let pid_file = folder.path().join("pid");
    let started = Instant::now();
    let finished = run_limited(LimitedRun {
        command: Command::new("sh")
            .arg("-c")
            .arg(format!("echo $$ > '{}'; exec sleep 30", pid_file.display())),
        input: b"unread input".repeat(100_000),
        deadline: Instant::now() + Duration::from_millis(500),
    })
    .expect("sh runs");

    assert!(matches!(finished, Finished::OutOfTime));
    assert!(started.elapsed() < Duration::from_secs(10));
    let pid = fs::read_to_string(&pid_file).expect("pid file");
    let alive = Command::new("kill")
        .args(["-0", pid.trim()])
        .status()
        .expect("kill runs");
    assert!(!alive.success(), "the stopped command is still running");
}
