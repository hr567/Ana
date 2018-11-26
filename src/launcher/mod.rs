use std::env;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

mod limit;
mod report;

pub use self::{
    limit::Limit,
    report::{LaunchReport, LrunExceed, LrunReport},
};

fn create_empty_lrun_log_file(id: &str) -> PathBuf {
    let mut lrun_log = env::temp_dir();
    lrun_log.push(id);
    lrun_log.set_extension("log");
    lrun_log
}

pub fn launch(executable_file: &Path, input: &str, limit: &Limit) -> (LaunchReport, LrunReport) {
    let lrun_log = create_empty_lrun_log_file(
        executable_file
            .file_name()
            .unwrap()
            .to_str()
            .expect("The judge id is not a hex number and cause the launcher crashing"),
    );

    let mut child = Command::new("sudo") // lrun need root user to be executed
        .arg("bash")
        .arg("-c")
        .arg(format!(
            "{} --max-cpu-time {} --max-real-time {} --max-memory {} --network false {} 3> {}",
            "lrun --uid 65534 --gid 65534", // 65534 is the id of nobody on my computer
            limit.time,
            limit.time + 0.1,
            limit.memory * 1024.0 * 1024.0,
            executable_file.to_str().unwrap(),
            lrun_log.to_str().unwrap(),
        ))
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
        .expect("Error when executing the program");

    assert!(output.status.success(), "lrun crashed! Why?");

    let report = LrunReport::from_log_file(&lrun_log);

    (
        match report.exceed {
            LrunExceed::Pass => {
                if report.exit_code == 0 {
                    if let Ok(output) = String::from_utf8(output.stdout) {
                        LaunchReport::Pass(output)
                    } else {
                        LaunchReport::RE
                    }
                } else {
                    LaunchReport::RE
                }
            }
            LrunExceed::CpuTime | LrunExceed::RealTime => LaunchReport::TLE,
            LrunExceed::Memory => LaunchReport::MLE,
            LrunExceed::Output => LaunchReport::OLE,
        },
        report,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_launcher() {
        match launch(
            &Path::new("/bin/bash"),
            &"echo hello world",
            &Limit::new(1.0, 64.0),
        )
        .0
        {
            LaunchReport::Pass(output) => assert_eq!(output, "hello world\n"),
            _ => panic!("Failed when execute program"),
        }
    }

    #[test]
    fn test_memory_limit() {
        unimplemented!("TODO: How to test memory")
    }

    #[test]
    fn test_time_limit() {
        match launch(
            &Path::new("/bin/bash"),
            &"while true; do echo -n; done",
            &Limit::new(1.0, 64.0),
        )
        .0
        {
            LaunchReport::TLE => {}
            _ => panic!("Failed when test time limit"),
        }
    }

    #[test]
    fn test_runtime_error() {
        match launch(&Path::new("/bin/bash"), &"exit 1", &Limit::new(1.0, 64.0)).0 {
            LaunchReport::RE => {}
            _ => panic!("Failed when test time limit"),
        }
    }
}
