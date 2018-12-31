use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::path;
use std::sync;
use std::thread;

use super::{
    compare::Comparer,
    compiler::Compiler,
    launcher::{launch, LaunchResult},
    mtp::*,
};

const NS_PER_SEC: f64 = 1_000_000_000 as f64;
const BYTES_PER_MB: f64 = (1024 * 1024) as f64;

pub struct JudgeReport {
    pub id: String,
    pub index: usize,
    pub status: JudgeResult,
    pub time: u64,
    pub memory: u64,
}

impl JudgeReport {
    pub fn new(id: &str, index: usize, status: JudgeResult, time: u64, memory: u64) -> JudgeReport {
        JudgeReport {
            id: id.to_string(),
            index,
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

impl fmt::Display for JudgeResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                JudgeResult::AC => "AC",
                JudgeResult::CE => "CE",
                JudgeResult::MLE => "MLE",
                JudgeResult::OLE => "OLE",
                JudgeResult::RE => "RE",
                JudgeResult::TLE => "TLE",
                JudgeResult::WA => "WA",
            }
        )
    }
}

impl Into<ReportInfo> for JudgeReport {
    fn into(self) -> ReportInfo {
        ReportInfo {
            id: self.id,
            index: self.index,
            status: self.status.to_string(),
            time: self.time as f64 / NS_PER_SEC,
            memory: self.memory as f64 / BYTES_PER_MB,
        }
    }
}

fn current_judge_id() -> String {
    thread::current()
        .name()
        .expect("Unnamed thread. Please set thread name as judge id")
        .to_string()
}

fn current_judge_work_dir() -> Box<path::Path> {
    path::Path::new(&env::var("ANA_WORK_DIR").unwrap())
        .join(&current_judge_id())
        .into_boxed_path()
}

fn create_file(filename: &str) -> Box<path::Path> {
    current_judge_work_dir().join(filename).into_boxed_path()
}

fn prepare_problem(problem: &Problem) -> (u64, u64, &Vec<TestCase>, Option<Box<path::Path>>) {
    (
        (problem.time_limit * NS_PER_SEC) as u64,
        (problem.memory_limit * BYTES_PER_MB) as u64,
        &problem.test_cases,
        match problem.get_type() {
            ProblemType::Normal => None,
            ProblemType::Special => {
                let spj = create_file("spj");
                Compiler::compile(&problem.checker.language, &problem.checker.code, &spj)
                    .expect("Failed to build spj");
                Some(spj)
            }
        },
    )
}

fn prepare_test_case(test_case: &TestCase) -> (Box<path::Path>, Box<path::Path>) {
    let input_file = create_file("input");
    fs::write(&input_file, test_case.input.as_bytes())
        .expect("Cannot write input content to input file");

    let answer_file = create_file("answer");
    fs::write(&answer_file, test_case.answer.as_bytes())
        .expect("Cannot write answer content to answer file");

    (input_file, answer_file)
}

fn judge_per_test_case(
    executable_file: &path::Path,
    input_file: &path::Path,
    answer_file: &path::Path,
    time_limit: u64,
    memory_limit: u64,
    spj: &Option<&path::Path>,
) -> io::Result<(JudgeResult, u64, u64)> {
    let output_file = create_file("output");
    let report = launch(
        &current_judge_id(),
        &executable_file,
        &input_file,
        &output_file,
        time_limit,
        memory_limit,
    )?;
    let judge_result = match &report.status {
        LaunchResult::Pass => {
            if Comparer::check(&input_file, &output_file, &answer_file, &spj)? {
                JudgeResult::AC
            } else {
                JudgeResult::WA
            }
        }
        LaunchResult::RE => JudgeResult::RE,
        LaunchResult::MLE => JudgeResult::MLE,
        LaunchResult::TLE => JudgeResult::TLE,
        LaunchResult::OLE => JudgeResult::OLE,
    };
    Ok((judge_result, report.time, report.memory))
}

pub fn judge(judge_info: &JudgeInfo, sender: &sync::mpsc::Sender<JudgeReport>) {
    let executable_file = create_file("main");
    let compile_flag = Compiler::compile(
        &judge_info.source.language,
        &judge_info.source.code,
        &executable_file,
    )
    .expect("Ana compiler crash when compiling source");

    if !compile_flag {
        sender
            .send(JudgeReport::new(&judge_info.id, 0, JudgeResult::CE, 0, 0))
            .expect("Cannot send the result to receiver");
        return;
    }

    let (time_limit, memory_limit, test_cases, spj) = prepare_problem(&judge_info.problem);
    for (index, test_case) in test_cases.iter().enumerate() {
        let (input_file, answer_file) = prepare_test_case(test_case);
        let judge_result = judge_per_test_case(
            &executable_file,
            &input_file,
            &answer_file,
            time_limit,
            memory_limit,
            &match spj {
                Some(ref spj) => Some(spj.as_ref()),
                None => None,
            },
        )
        .expect("Failed when judging");

        sender
            .send(JudgeReport::new(
                &judge_info.id,
                index,
                judge_result.0,
                judge_result.1,
                judge_result.2,
            ))
            .expect("Cannot send the result to receiver");
    }
}
