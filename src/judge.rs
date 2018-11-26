use std::char::from_digit;
use std::env::temp_dir;
use std::path::{Path, PathBuf};
use std::sync;

use rand::prelude::*;

use super::{
    compare::diff,
    compiler::{CompileResult, Compiler},
    launcher::{launch, LaunchResult, Limit},
    mtp::{Problem, Source, TestCase},
};

pub struct JudgeReport {
    pub status: JudgeResult,
    pub time: f64,
    pub memory: u64,
}

impl JudgeReport {
    pub fn new(status: JudgeResult, time: f64, memory: u64) -> JudgeReport {
        JudgeReport {
            status,
            time,
            memory,
        }
    }
}

pub enum JudgeResult {
    CE,
    AC,
    WA,
    TLE,
    MLE,
    OLE,
    RE,
}

impl JudgeResult {
    pub fn to_str(&self) -> &'static str {
        match self {
            JudgeResult::AC => "AC",
            JudgeResult::CE => "CE",
            JudgeResult::MLE => "MLE",
            JudgeResult::OLE => "OLE",
            JudgeResult::RE => "RE",
            JudgeResult::TLE => "TLE",
            JudgeResult::WA => "WA",
        }
    }
}

fn create_executable_filename() -> PathBuf {
    let mut executable_file = temp_dir();
    let filename = {
        let mut res = String::new();
        let mut rand_num: u32 = random();
        for _ in 0..8 {
            res.push(from_digit(rand_num & 0x0000_000f, 16).unwrap());
            rand_num >>= 4;
        }
        res
    };
    executable_file.push(filename);
    executable_file.set_extension("exe");
    executable_file
}

fn judge_per_test_case(executable_file: &Path, test_case: &TestCase, limit: &Limit) -> JudgeReport {
    let (result, report) = launch(executable_file, &test_case.input, &limit);
    JudgeReport::new(
        match result {
            LaunchResult::Pass(output) => {
                if !diff(&output, &test_case.answer) {
                    JudgeResult::AC
                } else {
                    JudgeResult::WA
                }
            }
            LaunchResult::RE => JudgeResult::RE,
            LaunchResult::MLE => JudgeResult::MLE,
            LaunchResult::TLE => JudgeResult::TLE,
            LaunchResult::OLE => JudgeResult::OLE,
        },
        report.cpu_time,
        report.memory,
    )
}

pub fn judge(source: &Source, problem: &Problem, sender: &sync::mpsc::Sender<JudgeReport>) {
    let executable_file = create_executable_filename();
    match Compiler::compile(&source.language, &source.code, &executable_file) {
        CompileResult::Pass => {
            let limit = Limit::new(problem.time_limit, problem.memory_limit);
            for test_case in &problem.test_cases {
                let judge_result = judge_per_test_case(&executable_file, test_case, &limit);
                sender
                    .send(judge_result)
                    .expect("Cannot send the result to receiver");
            }
        }
        CompileResult::CE => sender
            .send(JudgeReport::new(JudgeResult::CE, 0.0, 0))
            .unwrap(),
    };
}
