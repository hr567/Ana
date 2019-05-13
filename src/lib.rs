/// A high performance framework for building judge system
use std::iter;
use std::path;
use std::time;

use log::*;

pub mod communicator;
pub mod compiler;
pub mod diff;
pub mod mtp;
pub mod runner;
pub mod threadpool;
pub mod workspace;

use communicator::*;
use workspace::*;

pub struct Ana<T: Receiver, U: Sender> {
    max_threads: usize,
    task_receiver: TaskReceiver<T>,
    report_sender: ReportSender<U>,
}

impl<T: Receiver + Send + 'static, U: Sender + Send + 'static> Ana<T, U> {
    pub fn new(max_threads: usize, task_receiver: T, report_sender: U) -> Ana<T, U> {
        Ana {
            max_threads,
            task_receiver: task_receiver.into(),
            report_sender: report_sender.into(),
        }
    }

    pub fn start(self) {
        let Ana {
            max_threads,
            task_receiver,
            report_sender,
        } = self;

        let thread_pool = threadpool::ThreadPool::new(max_threads);

        loop {
            let judge_task = match task_receiver.receive() {
                Ok(task) => task,
                Err(communicator::Error::Network) => panic!("Network error"),
                Err(communicator::Error::Data) => panic!("Data error"),
                Err(communicator::Error::EOF) => break,
            };

            let report_sender = report_sender.clone();
            thread_pool.spawn(move || {
                for judge_report in judge(judge_task) {
                    report_sender
                        .send(judge_report)
                        .expect("Failed to send judge report");
                }
            });
        }
    }
}

/// Judge the task and generate a list of reports
fn judge(judge_task: mtp::JudgeTask) -> Box<dyn iter::Iterator<Item = mtp::JudgeReport>> {
    debug!("[Start] Judge task :{}", &judge_task.id);

    let work_dir = Workspace::new();
    work_dir.prepare_judge_task(&judge_task);

    debug!("Create work directory at {:?}", work_dir.as_ref());

    let mtp::JudgeTask {
        id,
        source,
        problem,
    } = judge_task;

    debug!("[Start] Compiling source code");
    let compile_success = {
        let mtp::Source {
            language: source_language,
            ..
        } = source;
        compiler::compile(
            &source_language,
            &work_dir.source_file(),
            &work_dir.runtime_dir().executable_file(),
        )
    };
    debug!("[Done] Compiling source code");

    if !compile_success {
        let ce_report = mtp::JudgeReport::new(&id, 0, mtp::JudgeResult::CE, 0, 0);
        return Box::new(iter::once(ce_report));
    }

    match problem {
        mtp::Problem::Normal {
            time_limit,
            memory_limit,
            ..
        } => {
            let time_limit = time::Duration::from_nanos(time_limit);
            let memory_limit = memory_limit as usize;

            let reports = work_dir
                .problem_dir()
                .test_case_dirs()
                .into_iter()
                .enumerate()
                .map(move |(index, test_case_dir)| {
                    debug!("Testing test case #{}", index);
                    let runner_report = runner::run(
                        Some(work_dir.runtime_dir()),
                        "/main",
                        test_case_dir.input_file(),
                        test_case_dir.output_file(),
                        time_limit,
                        memory_limit,
                    );
                    let report = generate_normal_problem_report(
                        &id,
                        time_limit,
                        memory_limit,
                        &work_dir.problem_dir(),
                        index,
                        runner_report,
                    );
                    debug!("[#{}] Generated report: {:?}", index, &report);
                    report
                });
            Box::new(reports)
        }
        mtp::Problem::Special {
            time_limit,
            memory_limit,
            spj,
            ..
        } => {
            let time_limit = time::Duration::from_nanos(time_limit);
            let memory_limit = memory_limit as usize;

            let spj_compile_success = {
                let mtp::Source {
                    language: spj_source_language,
                    ..
                } = spj;
                compiler::compile(
                    &spj_source_language,
                    &work_dir.problem_dir().spj_source(),
                    &work_dir.problem_dir().spj_file(),
                )
            };
            assert!(spj_compile_success, "Failed to compile spj");

            let reports = work_dir
                .problem_dir()
                .test_case_dirs()
                .into_iter()
                .enumerate()
                .map(move |(index, test_case_dir)| {
                    debug!("Testing test case #{}", index);
                    let runner_report = runner::run(
                        Some(work_dir.runtime_dir()),
                        "/main",
                        test_case_dir.input_file(),
                        test_case_dir.output_file(),
                        time_limit,
                        memory_limit,
                    );
                    let report = generate_special_judge_problem_report(
                        &id,
                        time_limit,
                        memory_limit,
                        &work_dir.problem_dir(),
                        index,
                        runner_report,
                    );
                    debug!("[#{}] Generated report: {:?}", index, &report);
                    report
                });
            Box::new(reports)
        }
    }
}

/// Generate the judge report using given data.
/// The order of different cases is important
/// because TLE or RE may be caused by MLE.
/// So check memory usage first.
/// If the cpu time usage being much smaller than
/// real time usage means that there are too many
/// threads working in one time or the program use sleep.
fn generate_normal_problem_report(
    id: &str,
    time_limit: time::Duration,
    memory_limit: usize,
    problem_dir: &path::Path,
    test_case_index: usize,
    runner_report: runner::RunnerReport,
) -> mtp::JudgeReport {
    let case_dir = &problem_dir.test_case_dirs()[test_case_index];

    let runner::RunnerReport {
        exit_success,
        real_time_usage,
        cpu_time_usage,
        memory_usage,
        tle_flag,
        mle_flag,
    } = runner_report;

    let status = if mle_flag || memory_usage >= memory_limit {
        mtp::JudgeResult::MLE
    } else if tle_flag || real_time_usage >= time_limit * 2 {
        mtp::JudgeResult::TLE
    } else if !exit_success {
        mtp::JudgeResult::RE
    } else if diff::check(&case_dir.output_file(), &case_dir.answer_file()).unwrap_or(false) {
        mtp::JudgeResult::AC
    } else {
        mtp::JudgeResult::WA
    };

    mtp::JudgeReport::new(
        &id,
        test_case_index,
        status,
        cpu_time_usage.as_nanos() as u64,
        memory_usage as u64,
    )
}

fn generate_special_judge_problem_report(
    id: &str,
    time_limit: time::Duration,
    memory_limit: usize,
    problem_dir: &path::Path,
    test_case_index: usize,
    runner_report: runner::RunnerReport,
) -> mtp::JudgeReport {
    let case_dir = &problem_dir.test_case_dirs()[test_case_index];

    let runner::RunnerReport {
        exit_success,
        real_time_usage,
        cpu_time_usage,
        memory_usage,
        tle_flag,
        mle_flag,
    } = runner_report;
    let status = if mle_flag || memory_usage >= memory_limit {
        mtp::JudgeResult::MLE
    } else if tle_flag || real_time_usage >= time_limit * 2 {
        mtp::JudgeResult::TLE
    } else if !exit_success {
        mtp::JudgeResult::RE
    } else if diff::check_with_spj(
        &case_dir.input_file(),
        &case_dir.output_file(),
        &case_dir.answer_file(),
        &problem_dir.spj_file(),
    )
    .unwrap_or(false)
    {
        mtp::JudgeResult::AC
    } else {
        mtp::JudgeResult::WA
    };
    mtp::JudgeReport::new(
        &id,
        test_case_index,
        status,
        cpu_time_usage.as_nanos() as u64,
        memory_usage as u64,
    )
}
