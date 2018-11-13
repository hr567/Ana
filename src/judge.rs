use std::char::from_digit;
use std::env::temp_dir;
use std::path::Path;
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
    AC,
    WA,
    TLE,
    MLE,
    OLE,
    RE,
}

pub fn judge(
    language: &str,
    source_code: &str,
    problem: &Problem,
    sender: sync::mpsc::Sender<JudgeResult>,
) {
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

    match Compiler::compile(language, source_code, &executable_file) {
        CompileResult::Pass => {
            for test_case in &problem.test_cases {
                let limit = Limit::new(problem.time_limit, problem.memory_limit);
                let judge_result = judge_per_test_case(&executable_file, test_case, &limit);
                sender
                    .send(judge_result)
                    .expect("Cannot send the result to receiver");
            }
        }

        CompileResult::CE => {
            sender.send(JudgeResult::CE).unwrap();
        }
    };
}

fn judge_per_test_case(executable_file: &Path, test_case: &TestCase, limit: &Limit) -> JudgeResult {
    match launch(executable_file, test_case.input.as_str(), limit) {
        LaunchResult::Pass(output, _lrun_report) => {
            match compare(output.as_str(), test_case.answer.as_str()) {
                CompareResult::AC => JudgeResult::AC,
                CompareResult::WA => JudgeResult::WA,
            }
        }
        LaunchResult::RE => JudgeResult::RE,
        LaunchResult::MLE => JudgeResult::MLE,
        LaunchResult::TLE => JudgeResult::TLE,
        LaunchResult::OLE => JudgeResult::OLE,
    }
}
