use std::env;
use std::fs;
use std::io;
use std::path;
use std::sync;
use std::thread;

use super::{
    compare::Comparer,
    compiler::Compiler,
    launcher::{launch, LaunchResult},
    mtp::{JudgeInfo, Problem, ProblemType, TestCase},
};

mod structure;

pub use self::structure::{JudgeReport, JudgeResult};

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
        (problem.time_limit * 1000.0 * 1000.0) as u64, // Convert to us
        (problem.memory_limit * 1024.0 * 1024.0) as u64, // Convert to bytes
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
) -> io::Result<JudgeReport> {
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
    Ok(JudgeReport::new(judge_result, report.time, report.memory))
}

pub fn judge(judge_info: &JudgeInfo, sender: &sync::mpsc::Sender<JudgeReport>) {
    let executable_file = create_file("main");
    if !Compiler::compile(
        &judge_info.source.language,
        &judge_info.source.code,
        &executable_file,
    )
    .expect("Self error when compiling source")
    {
        sender
            .send(JudgeReport::new(JudgeResult::CE, 0, 0))
            .expect("Cannot send the result to receiver");
        return;
    };

    let (time_limit, memory_limit, test_cases, spj) = prepare_problem(&judge_info.problem);
    for test_case in test_cases {
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
            .send(judge_result)
            .expect("Cannot send the result to receiver");
    }
}
