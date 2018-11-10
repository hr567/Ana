// TODO: Rewrite the launcher module

use std::io::prelude::*;
use std::path::Path;
use std::process::{Command, Output, Stdio};

#[derive(PartialEq, Debug)]
pub enum LaunchResult {
    Pass(Output),
    TLE,
    MLE,
    RE,
}

pub struct Limit {
    pub time: f64,   // Sec
    pub memory: f64, // MB
}

impl Limit {
    pub fn new(time: f64, memory: f64) -> Self {
        Limit { time, memory }
    }
}

pub struct Launcher {}

impl Launcher {
    pub fn new() -> Self {
        Launcher {}
    }

    pub fn run(&self, executable_file: &Path, input: &str, limit: &Limit) -> LaunchResult {
        let mut child = Command::new("timeout")
            .arg(format!("{}", limit.time))
            .arg(executable_file)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn chile process");

        child
            .stdin
            .as_mut()
            .expect("Failed to open stdin")
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin");

        let output = child
            .wait_with_output()
            .expect("Failed to wait on child with output");

        if output.status.success() {
            LaunchResult::Pass(output)
        } else {
            let status_code = output.status.code().unwrap();
            // TODO: Add TLE, MLE
            match status_code {
                124 => LaunchResult::TLE,
                _ => LaunchResult::RE,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_launcher() {
        let launcher = Launcher::new();
        match launcher.run(
            &Path::new("/bin/bash"),
            &"echo hello world",
            &Limit::new(1.0, 64.0),
        ) {
            LaunchResult::Pass(output) => {
                assert_eq!(output.stdout.as_slice(), "hello world\n".as_bytes())
            }
            _ => panic!("Failed when execute program"),
        }
    }

    // #[test]
    // fn test_memory_limit() {
    //     unimplemented!()
    // }

    #[test]
    fn test_time_limit() {
        let launcher = Launcher::new();
        match launcher.run(
            &Path::new("/bin/bash"),
            &"while true; do echo -n; done",
            &Limit::new(1.0, 64.0),
        ) {
            LaunchResult::TLE => {}
            _ => panic!("Failed when test time limit"),
        }
    }
}
