use std::char::from_digit;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use rand::prelude::*;

pub enum LaunchResult {
    Pass(String, LrunResult),
    TLE,
    MLE,
    OLE,
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

pub enum LrunExceed {
    Pass,
    CpuTime,
    RealTime,
    Memory,
    Output,
}

pub struct LrunResult {
    pub memory: u64,    // Bytes
    pub cpu_time: f64,  // Seconds
    pub real_time: f64, // Seconds
    pub signaled: i32,
    pub exit_code: i32,
    pub term_sig: i32,
    pub exceed: LrunExceed,
}

impl LrunResult {
    fn from_log_file(lrun_log_path: &Path) -> LrunResult {
        let lrun_log = {
            let mut res = String::new();
            File::open(lrun_log_path)
                .expect("Cannot open lrun log")
                .read_to_string(&mut res)
                .expect("Cannot read the lrun log");
            res
        };
        let lrun_result: Vec<&str> = lrun_log
            .trim()
            .split("\n")
            .map(|s| s.trim().split_whitespace().collect::<Vec<&str>>()[1])
            .collect();

        LrunResult {
            memory: lrun_result[0].parse().unwrap(),
            cpu_time: lrun_result[1].parse().unwrap(),
            real_time: lrun_result[2].parse().unwrap(),
            signaled: lrun_result[3].parse().unwrap(),
            exit_code: lrun_result[4].parse().unwrap(),
            term_sig: lrun_result[5].parse().unwrap(),
            exceed: match lrun_result[6] {
                "none" => LrunExceed::Pass,
                "CPU_TIME" => LrunExceed::CpuTime,
                "REAL_TIME" => LrunExceed::RealTime,
                "MEMORY" => LrunExceed::Memory,
                "OUTPUT" => LrunExceed::Output,
                _ => panic!("Unknown type of exceed"),
            },
        }
    }
}

fn create_empty_lrun_log_file() -> PathBuf {
    let mut lrun_log = env::temp_dir();
    let filename = {
        let mut res = String::new();
        let mut rand_num: u32 = random();
        for _ in 0..8 {
            res.push(from_digit(rand_num & 0x0000000f, 16).unwrap());
            rand_num >>= 4;
        }
        res
    };
    lrun_log.push(filename);
    lrun_log.set_extension("log");
    lrun_log
}

pub fn launch(executable_file: &Path, input: &str, limit: &Limit) -> LaunchResult {
    let lrun_log = create_empty_lrun_log_file();

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

    let lrun_result = LrunResult::from_log_file(&lrun_log);
    match lrun_result.exceed {
        LrunExceed::Pass => {
            if lrun_result.exit_code == 0 {
                if let Ok(output) = String::from_utf8(output.stdout) {
                    LaunchResult::Pass(output, lrun_result)
                } else {
                    LaunchResult::RE
                }
            } else {
                LaunchResult::RE
            }
        }
        LrunExceed::CpuTime | LrunExceed::RealTime => LaunchResult::TLE,
        LrunExceed::Memory => LaunchResult::MLE,
        LrunExceed::Output => LaunchResult::OLE,
    }
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
        ) {
            LaunchResult::Pass(output, _lrun_report) => assert_eq!(output, "hello world\n"),
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
        ) {
            LaunchResult::TLE => {}
            _ => panic!("Failed when test time limit"),
        }
    }

    #[test]
    fn test_runtime_error() {
        match launch(&Path::new("/bin/bash"), &"exit 1", &Limit::new(1.0, 64.0)) {
            LaunchResult::RE => {}
            _ => panic!("Failed when test time limit"),
        }
    }
}
