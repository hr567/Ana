use std::fs;
use std::io::prelude::*;
use std::path;

pub enum LaunchReport {
    Pass(Box<path::Path>),
    TLE,
    MLE,
    OLE,
    RE,
}

pub enum LrunExceed {
    Pass,
    CpuTime,
    RealTime,
    Memory,
    Output,
}

pub struct LrunReport {
    pub memory: u64,    // Bytes
    pub cpu_time: f64,  // Seconds
    pub real_time: f64, // Seconds
    pub signaled: i32,
    pub exit_code: i32,
    pub term_sig: i32,
    pub exceed: LrunExceed,
}

impl LrunReport {
    pub fn from_log_file(lrun_log_path: &path::Path) -> LrunReport {
        let lrun_log = {
            let mut res = String::new();
            fs::File::open(lrun_log_path)
                .expect("Cannot open lrun log")
                .read_to_string(&mut res)
                .expect("Cannot read the lrun log");
            res
        };
        let lrun_result: Vec<&str> = lrun_log
            .trim()
            .split('\n')
            .map(|s| s.trim().split_whitespace().collect::<Vec<&str>>()[1])
            .collect();

        LrunReport {
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
