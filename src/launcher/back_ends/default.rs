use std::io::prelude::*;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use super::Launcher;

struct DefaultLauncher<'a> {
    executable_file: &'a Path,
    input: &'a str,
}

impl<'a> DefaultLauncher<'a> {
    fn new(executable_file: &'a Path, input: &'a str) -> Self {
        DefaultLauncher {
            executable_file,
            input,
        }
    }
}

impl<'a> Launcher for DefaultLauncher<'a> {
    fn run(&self) -> Output {
        let mut child = Command::new(self.executable_file)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn chile process");

        let input = child.stdin.as_mut().expect("Failed to open stdin");
        input
            .write_all(self.input.as_bytes())
            .expect("Failed to write to stdin");
        drop(input);

        let output = child
            .wait_with_output()
            .expect("Failed to wait on child with output");
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_launcher() {
        let launcher = DefaultLauncher::new(Path::new("/bin/bash"), &"echo hello world");
        let output = launcher.run();
        assert_eq!(output.stdout.as_slice(), "hello world\n".as_bytes());
    }
}
