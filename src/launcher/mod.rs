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
    pub time: f64,
    pub memory: f64,
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

    pub fn run(&self, executable_file: &Path, input: &str) -> LaunchResult {
        let mut child = Command::new(executable_file)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn chile process");

        {
            child
                .stdin
                .as_mut()
                .expect("Failed to open stdin")
                .write_all(input.as_bytes())
                .expect("Failed to write to stdin");
        }

        let output = child
            .wait_with_output()
            .expect("Failed to wait on child with output");

        LaunchResult::Pass(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_launcher() {
        let launcher = Launcher::new();
        match launcher.run(&Path::new("/bin/bash"), &"echo hello world") {
            LaunchResult::Pass(output) => {
                assert_eq!(output.stdout.as_slice(), "hello world\n".as_bytes())
            }
            _ => panic!("Failed when execute program"),
        }
    }
}
