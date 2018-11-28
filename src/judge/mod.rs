use std::env;
use std::fs;
use std::io::prelude::*;
use std::path;
use std::sync;

use super::{
    compare::Comparer,
    compiler::Compiler,
    launcher::{launch, LaunchReport, Limit},
    mtp::{JudgeInfo, Problem, ProblemType, TestCase},
};

mod structure;

pub use self::structure::{JudgeReport, JudgeResult};

fn create_executable_filename(id: &str) -> path::PathBuf {
    let mut executable_file = path::PathBuf::from(env::var("ANA_WORK_DIR").unwrap());
    executable_file.push(id);
    executable_file.set_extension("exe");
    executable_file
}

fn prepare_problem(problem: &Problem) -> (Limit, &Vec<TestCase>, Option<Box<path::Path>>) {
    (
        Limit::new(problem.time_limit, problem.memory_limit),
        &problem.test_cases,
        match problem.get_type() {
            ProblemType::Normal => None,
            ProblemType::Special => {
                let spj = create_executable_filename("spj");
                Compiler::compile(&problem.checker.language, &problem.checker.code, &spj)
                    .expect("Failed to build spj");
                Some(spj.into_boxed_path())
            }
        },
    )
}

fn prepare_test_case(test_case: &TestCase) -> (Box<path::Path>, Box<path::Path>) {
    let (mut input_file, mut answer_file) = (
        path::PathBuf::from(env::var("ANA_WORK_DIR").unwrap()),
        path::PathBuf::from(env::var("ANA_WORK_DIR").unwrap()),
    );

    input_file.push(env::var("ANA_JUDGE_ID").unwrap());
    input_file.set_extension("in");
    fs::File::create(&input_file)
        .expect("Cannot create input file")
        .write_all(test_case.input.as_bytes())
        .expect("Cannot write input content to input file");

    answer_file.push(env::var("ANA_JUDGE_ID").unwrap());
    answer_file.set_extension("ans");
    fs::File::create(&answer_file)
        .expect("Cannot create answer file")
        .write_all(test_case.answer.as_bytes())
        .expect("Cannot write answer content to answer file");

    (input_file.into_boxed_path(), answer_file.into_boxed_path())
}

fn judge_per_test_case(
    executable_file: &path::Path,
    input_file: &path::Path,
    answer_file: &path::Path,
    limit: &Limit,
    spj: &Option<&path::Path>,
) -> JudgeReport {
    let (result, report) = launch(executable_file, input_file, limit);
    let judge_result = match result {
        LaunchReport::Pass(output_file) => {
            if Comparer::check(&input_file, &output_file, &answer_file, spj) {
                JudgeResult::AC
            } else {
                JudgeResult::WA
            }
        }
        LaunchReport::RE => JudgeResult::RE,
        LaunchReport::MLE => JudgeResult::MLE,
        LaunchReport::TLE => JudgeResult::TLE,
        LaunchReport::OLE => JudgeResult::OLE,
    };
    JudgeReport::new(judge_result, report.cpu_time, report.memory)
}

pub fn judge(judge_info: &JudgeInfo, sender: &sync::mpsc::Sender<JudgeReport>) {
    let executable_file = create_executable_filename(&judge_info.id);
    if Compiler::compile(
        &judge_info.source.language,
        &judge_info.source.code,
        &executable_file,
    )
    .is_err()
    {
        sender
            .send(JudgeReport::new(JudgeResult::CE, 0.0, 0))
            .expect("Cannot send the result to receiver");
        return;
    };

    let (limit, test_cases, spj) = prepare_problem(&judge_info.problem);
    for test_case in test_cases {
        let (input_file, answer_file) = prepare_test_case(test_case);
        let judge_result = judge_per_test_case(
            &executable_file,
            &input_file,
            &answer_file,
            &limit,
            &match spj {
                Some(ref spj) => Some(spj.as_ref()),
                None => None,
            },
        );
        sender
            .send(judge_result)
            .expect("Cannot send the result to receiver");
    }
}
