use std::env::temp_dir;
use std::path::{Path, PathBuf};
use std::sync;

use super::{
    compare::diff,
    compiler::{CompileResult, Compiler},
    launcher::{launch, LaunchReport, Limit},
    mtp::{JudgeInfo, TestCase},
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

fn create_executable_filename(id: &str) -> PathBuf {
    let mut executable_file = temp_dir();
    executable_file.push(id);
    executable_file.set_extension("exe");
    executable_file
}

fn judge_per_test_case(executable_file: &Path, test_case: &TestCase, limit: &Limit) -> JudgeReport {
    let (result, report) = launch(executable_file, &test_case.input, &limit);
    JudgeReport::new(
        match result {
            LaunchReport::Pass(output) => {
                if !diff(&output, &test_case.answer) {
                    JudgeResult::AC
                } else {
                    JudgeResult::WA
                }
            }
            LaunchReport::RE => JudgeResult::RE,
            LaunchReport::MLE => JudgeResult::MLE,
            LaunchReport::TLE => JudgeResult::TLE,
            LaunchReport::OLE => JudgeResult::OLE,
        },
        report.cpu_time,
        report.memory,
    )
}

pub fn judge(judge_info: &JudgeInfo, sender: &sync::mpsc::Sender<JudgeReport>) {
    let executable_file = create_executable_filename(&judge_info.id);
    match Compiler::compile(
        &judge_info.source.language,
        &judge_info.source.code,
        &executable_file,
    ) {
        CompileResult::Pass => {
            let limit = Limit::new(
                judge_info.problem.time_limit,
                judge_info.problem.memory_limit,
            );
            for test_case in &judge_info.problem.test_cases {
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
