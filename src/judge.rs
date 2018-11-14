use std::char::from_digit;
use std::env::temp_dir;
use std::path::{Path, PathBuf};
use std::sync;

use rand::prelude::*;

use super::{
    compare::{compare, CompareResult},
    compiler::{CompileResult, Compiler},
    launcher::{launch, LaunchResult, Limit},
    mtp::{Problem, TestCase},
};

pub enum JudgeResult {
    CE,
    AC(f64, u64), // time, memory
    WA(f64, u64),
    TLE(f64, u64),
    MLE(f64, u64),
    OLE(f64, u64),
    RE(f64, u64),
}

fn create_executable_filename() -> PathBuf {
    let mut executable_file = temp_dir();
    let filename = {
        let mut res = String::new();
        let mut rand_num: u32 = random();
        for _ in 0..8 {
            res.push(from_digit(rand_num & 0x0000000f, 16).unwrap());
            rand_num >>= 4;
        }
        res
    };
    executable_file.push(filename);
    executable_file.set_extension("exe");
    executable_file
}

fn judge_per_test_case(executable_file: &Path, test_case: &TestCase, limit: &Limit) -> JudgeResult {
    match launch(executable_file, test_case.input.as_str(), limit) {
        LaunchResult::Pass(output, report) => {
            match compare(output.as_str(), test_case.answer.as_str()) {
                CompareResult::AC => JudgeResult::AC(report.cpu_time, report.memory),
                CompareResult::WA => JudgeResult::WA(report.cpu_time, report.memory),
            }
        }
        LaunchResult::RE(report) => JudgeResult::RE(report.cpu_time, report.memory),
        LaunchResult::MLE(report) => JudgeResult::MLE(report.cpu_time, report.memory),
        LaunchResult::TLE(report) => JudgeResult::TLE(report.cpu_time, report.memory),
        LaunchResult::OLE(report) => JudgeResult::OLE(report.cpu_time, report.memory),
    }
}

pub fn judge(
    language: &str,
    source_code: &str,
    problem: &Problem,
    sender: sync::mpsc::Sender<JudgeResult>,
) {
    let executable_file = create_executable_filename();
    match Compiler::compile(language, source_code, &executable_file) {
        CompileResult::Pass => {
            let limit = Limit::new(problem.time_limit, problem.memory_limit);
            for test_case in &problem.test_cases {
                sender
                    .send(judge_per_test_case(&executable_file, test_case, &limit))
                    .expect("Cannot send the result to receiver");
            }
        }
        CompileResult::CE => sender.send(JudgeResult::CE).unwrap(),
    };
}
