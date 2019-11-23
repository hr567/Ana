use super::*;

use std::fs::{self, File};
use std::io::{self, prelude::*};
use std::process::Command;
use std::time::Duration;
use std::time::Instant;

use cgroup::CommandExt as _;
use tempfile;

const PROGRAM: &str = r#"/bin/sh"#;
const TIMEOUT_INPUT_CONTENT: &str = r#"sleep 100"#;
const INPUT_CONTENT: &str = r#"echo -n "hello, world""#;
const ANSWER_CONTENT: &str = r#"hello, world"#;

macro_rules! input_file {
    () => {{
        let mut input_file = tempfile::Builder::new().suffix(".in").tempfile()?;
        input_file.write_all(INPUT_CONTENT.as_bytes())?;
        input_file.into_temp_path()
    }};
}

macro_rules! output_file {
    () => {{
        tempfile::Builder::new()
            .suffix(".out")
            .tempfile()?
            .into_temp_path()
    }};
}

macro_rules! timeout_input_file {
    () => {{
        let mut input_file = tempfile::Builder::new().suffix(".in").tempfile()?;
        input_file.write_all(TIMEOUT_INPUT_CONTENT.as_bytes())?;
        input_file.into_temp_path()
    }};
}

macro_rules! cg_ctx {
    () => {{
        let cg_ctx = cgroup::Builder::new().build()?;

        let cpu_controller = cg_ctx.cpu_controller().unwrap();
        let memory_controller = cg_ctx.memory_controller().unwrap();

        let real_time = Duration::from_secs(2);
        let cpu_time = Duration::from_secs(1);

        let period = Duration::from_secs(1);
        let quota = {
            let real_time = real_time.as_micros() as u32;
            let cpu_time = cpu_time.as_micros() as u32;
            period * cpu_time / real_time
        };

        cpu_controller.period().write(&period)?;
        cpu_controller.quota().write(&quota)?;
        memory_controller
            .limit_in_bytes()
            .write(&(16 * 1024 * 1024))?;

        cg_ctx
    }};
}

#[test]
fn test_cgroup() -> io::Result<()> {
    let input_file = input_file!();
    let output_file = output_file!();
    let cg_ctx = cg_ctx!();

    let start_time = Instant::now();
    let exit_status = Command::new(&PROGRAM)
        .stdin(File::open(&input_file)?)
        .stdout(File::create(&output_file)?)
        .cgroup(cg_ctx.clone())
        .spawn()?
        .wait()?;
    let cpu_usage = cg_ctx.cpuacct_controller().unwrap().usage()?;
    let time_usage = start_time.elapsed();
    let memory_usage = cg_ctx.memory_controller().unwrap().max_usage_in_bytes()?;

    assert!(exit_status.success());
    assert_eq!(fs::read(&output_file)?, ANSWER_CONTENT.as_bytes());

    assert_ne!(cpu_usage, Duration::from_secs(0));
    assert_ne!(time_usage, Duration::from_secs(0));
    assert_ne!(memory_usage, 0);

    assert!(cpu_usage <= Duration::from_secs(2));
    assert!(time_usage <= Duration::from_secs(1));
    assert!(memory_usage <= 16 * 1024 * 1024);

    Ok(())
}

#[test]
fn test_chroot() -> io::Result<()> {
    let input_file = input_file!();
    let output_file = output_file!();

    let exit_status = Command::new(&PROGRAM)
        .stdin(File::open(&input_file)?)
        .stdout(File::create(&output_file)?)
        .chroot("/")
        .spawn()?
        .wait()?;

    assert!(exit_status.success());
    assert_eq!(fs::read(&output_file)?, ANSWER_CONTENT.as_bytes());

    Ok(())
}

#[test]
fn test_unshare() -> io::Result<()> {
    let input_file = input_file!();
    let output_file = output_file!();

    let exit_status = Command::new(&PROGRAM)
        .stdin(File::open(&input_file)?)
        .stdout(File::create(&output_file)?)
        .unshare_all_ns()
        .spawn()?
        .wait()?;

    assert!(exit_status.success());
    assert_eq!(fs::read(&output_file)?, ANSWER_CONTENT.as_bytes());

    Ok(())
}

#[test]
fn test_timeout() -> io::Result<()> {
    let input_file = timeout_input_file!();
    let output_file = output_file!();

    let exit_status = Command::new(&PROGRAM)
        .stdin(File::open(&input_file)?)
        .stdout(File::create(&output_file)?)
        .spawn()?
        .timeout(Duration::from_secs(1))?;

    assert!(!exit_status.success());

    Ok(())
}

#[cfg(feature = "seccomp")]
#[test]
#[should_panic]
fn test_seccomp() {
    unimplemented!("TODO: Add seccomp test")
}

#[cfg(feature = "cap-ng")]
#[test]
#[should_panic]
fn test_capabilities() {
    unimplemented!("TODO: Add capabilities test")
}
