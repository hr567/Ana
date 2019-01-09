use std::env;
use std::fs;
use std::io;
use std::path;

use super::{
    communicator::ReportSender,
    compare::Comparer,
    compiler::Compiler,
    launcher::{launch, LaunchResult},
    mtp::*,
};

const NS_PER_SEC: f64 = 1_000_000_000 as f64;
const BYTES_PER_MB: f64 = (1024 * 1024) as f64;

struct WorkDir {
    work_dir: Box<path::Path>,
}

impl WorkDir {
    pub fn new(id: &str) -> WorkDir {
        let ana_work_dir = &env::var("ANA_WORK_DIR")
            .unwrap_or_else(|_| env::temp_dir().to_str().unwrap().to_string());
        let work_dir = path::Path::new(&ana_work_dir).join(&id).into_boxed_path();
        fs::create_dir(&work_dir).expect("Failed to create work dir");
        WorkDir { work_dir }
    }

    pub fn create_file(&self, filename: &str) -> Box<path::Path> {
        self.work_dir.join(filename).into_boxed_path()
    }
}

impl Drop for WorkDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.work_dir).unwrap();
    }
}

fn prepare_problem<'a>(
    work_dir: &WorkDir,
    problem: &'a Problem,
) -> (u64, u64, &'a Vec<TestCase>, Option<Box<path::Path>>) {
    (
        (problem.time_limit * NS_PER_SEC) as u64,
        (problem.memory_limit * BYTES_PER_MB) as u64,
        &problem.test_cases,
        match problem.get_type() {
            ProblemType::Normal => None,
            ProblemType::Special => {
                let spj = work_dir.create_file("spj.exe");
                let spj_source_file = work_dir.create_file("spj");
                fs::write(&spj_source_file, &problem.checker.code)
                    .expect("Failed to write spj source code");
                Compiler::compile(&problem.checker.language, &spj_source_file, &spj)
                    .expect("Failed to build spj");
                Some(spj)
            }
        },
    )
}

fn prepare_test_case(
    work_dir: &WorkDir,
    test_case: &TestCase,
) -> (Box<path::Path>, Box<path::Path>) {
    let input_file = work_dir.create_file("input");
    fs::write(&input_file, test_case.input.as_bytes())
        .expect("Cannot write input content to input file");

    let answer_file = work_dir.create_file("answer");
    fs::write(&answer_file, test_case.answer.as_bytes())
        .expect("Cannot write answer content to answer file");

    (input_file, answer_file)
}

fn judge_per_test_case(
    work_dir: &WorkDir,
    judge_id: &str,
    executable_file: &path::Path,
    input_file: &path::Path,
    answer_file: &path::Path,
    time_limit: u64,
    memory_limit: u64,
    spj: &Option<&path::Path>,
) -> io::Result<(JudgeResult, u64, u64)> {
    let output_file = work_dir.create_file("output");
    let report = launch(
        &judge_id,
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

pub fn judge(judge_info: JudgeInfo, sender: impl ReportSender) {
    let work_dir = WorkDir::new(&judge_info.id);

    let executable_file = work_dir.create_file("main");
    let source_file = work_dir.create_file("source");
    fs::write(&source_file, &judge_info.source.code).expect("Failed to write source code");
    let compile_flag =
        Compiler::compile(&judge_info.source.language, &source_file, &executable_file)
            .expect("Ana compiler crash when compiling source");

    if !compile_flag {
        sender.send_report_information(ReportInfo::new(
            &judge_info.id,
            0,
            JudgeResult::CE,
            0.0,
            0.0,
        ));
        return;
    }

    let (mut summary_status, mut max_time_usage, mut max_memory_usage) = (JudgeResult::AC, 0, 0);

    let (time_limit, memory_limit, test_cases, spj) =
        prepare_problem(&work_dir, &judge_info.problem);
    for (index, test_case) in test_cases.iter().enumerate() {
        let (input_file, answer_file) = prepare_test_case(&work_dir, &test_case);
        let judge_result = judge_per_test_case(
            &work_dir,
            &judge_info.id,
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

        if let JudgeResult::AC = summary_status {
            summary_status = judge_result.0;
        }
        if judge_result.1 > max_time_usage {
            max_time_usage = judge_result.1;
        }
        if judge_result.2 > max_memory_usage {
            max_memory_usage = judge_result.2;
        }

        sender.send_report_information(ReportInfo::new(
            &judge_info.id,
            index,
            judge_result.0,
            judge_result.1 as f64 / NS_PER_SEC,
            judge_result.2 as f64 / BYTES_PER_MB,
        ));
    }

    sender.send_report_information(ReportInfo::new(
        &judge_info.id,
        judge_info.problem.len(),
        summary_status,
        max_time_usage as f64 / NS_PER_SEC,
        max_memory_usage as f64 / BYTES_PER_MB,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_work_dir() {
        let work_dir = WorkDir::new("test_work_dir");
        let work_dir_path = work_dir.work_dir.clone();
        assert!(work_dir.work_dir.exists());

        let file_a = work_dir.create_file("a");
        assert!(file_a.parent().unwrap().exists());
        assert!(!file_a.exists());

        let file_b = work_dir.create_file("b");
        assert!(file_b.parent().unwrap().exists());
        assert!(!file_b.exists());

        drop(work_dir);
        assert!(!work_dir_path.exists());
    }
}
