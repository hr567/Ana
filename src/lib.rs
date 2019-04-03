use std::path;
use std::sync;
use std::time;

use log::*;
use tokio::prelude::*;
use tokio_threadpool;

mod communicator;
mod compiler;
mod diff;
mod mtp;
mod runner;
mod workspace;

use communicator::*;
use workspace::*;

/// The entry of judging
/// Generate reports for every task from receiver and then send them
/// Should be called only once
pub fn start_judging<T, U>(
    judge_threads: usize,
    judge_receiver: impl Into<TaskReceiver<T>>,
    report_sender: impl Into<ReportSender<U>>,
) where
    T: Receiver + Send + 'static,
    U: Sender + Send + 'static,
{
    let judge_receiver = judge_receiver.into();
    let report_sender = report_sender.into();

    let pool = tokio_threadpool::Builder::new()
        .pool_size(judge_threads)
        .build();

    let server = judge_receiver.for_each(move |judge_task| {
        debug!("Received judge information: {:?}", &judge_task);
        let (tx, rx) = sync::mpsc::channel();
        pool.spawn(future::lazy(move || {
            judge(judge_task, tx);
            Ok(())
        }));
        for report in rx {
            report_sender
                .send(report)
                .unwrap_or_else(|_| error!("Failed to send report"));
        }
        Ok(())
    });

    tokio::run(server);
}

/// Judge the task and generate a list of reports
fn judge(judge_task: mtp::JudgeTask, report_sender: sync::mpsc::Sender<mtp::JudgeReport>) {
    debug!("[Start] Judge task :{}", &judge_task.id);

    let work_dir = WorkSpace::new();
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

    match problem {
        mtp::Problem::Normal {
            time_limit,
            memory_limit,
            ..
        } => {
            if !compile_success {
                let ce_report = mtp::JudgeReport::new(&id, 0, mtp::JudgeResult::CE, 0, 0);
                report_sender
                    .send(ce_report)
                    .expect("Failed to send report");
                return;
            }

            for (index, test_case_dir) in work_dir.problem_dir().test_case_dirs().iter().enumerate()
            {
                debug!("Testing test case #{}", index);
                let launch_result = runner::launch(
                    &work_dir.runtime_dir(),
                    &test_case_dir.input_file(),
                    &test_case_dir.output_file(),
                    time_limit,
                    memory_limit,
                )
                .wait()
                .expect("Failed to run compiled program");
                let report = generate_normal_problem_report(
                    &id,
                    time_limit,
                    memory_limit,
                    &work_dir.problem_dir(),
                    index,
                    launch_result,
                );
                debug!("[#{}] Generated report: {:?}", index, &report);
                report_sender.send(report).expect("Failed to send report");
            }
        }
        mtp::Problem::Special {
            time_limit,
            memory_limit,
            spj,
            ..
        } => {
            if !compile_success {
                let ce_report = mtp::JudgeReport::new(&id, 0, mtp::JudgeResult::CE, 0, 0);
                report_sender
                    .send(ce_report)
                    .expect("Failed to send report");
                return;
            }

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

            for (index, test_case_dir) in work_dir.problem_dir().test_case_dirs().iter().enumerate()
            {
                debug!("Testing test case #{}", index);
                let launch_result = runner::launch(
                    &work_dir.runtime_dir(),
                    &test_case_dir.input_file(),
                    &test_case_dir.output_file(),
                    time_limit,
                    memory_limit,
                )
                .wait()
                .expect("Failed to run compiled program");
                let report = generate_special_judge_problem_report(
                    &id,
                    time_limit,
                    memory_limit,
                    &work_dir.problem_dir(),
                    index,
                    launch_result,
                );
                debug!("[#{}] Generated report: {:?}", index, &report);
                report_sender.send(report).expect("Failed to send report");
            }
        }
    }
    debug!("[Done] Judge task");
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
    time_limit: u64,
    memory_limit: u64,
    problem_dir: &path::Path,
    test_case_index: usize,
    launch_result: runner::LaunchResult,
) -> mtp::JudgeReport {
    let case_dir = &problem_dir.test_case_dirs()[test_case_index];
    let output_file = case_dir.output_file();
    let answer_file = case_dir.answer_file();

    let runner::LaunchResult {
        exit_code,
        real_time_usage,
        cpu_time_usage,
        memory_usage,
        tle_flag,
        mle_flag,
    } = launch_result;
    let status = if mle_flag || memory_usage >= memory_limit {
        mtp::JudgeResult::MLE
    } else if tle_flag || real_time_usage >= time::Duration::from_nanos(time_limit / 2 * 3) {
        mtp::JudgeResult::TLE
    } else if exit_code != 0 {
        mtp::JudgeResult::RE
    } else if diff::check(&output_file, &answer_file).unwrap_or(false) {
        mtp::JudgeResult::AC
    } else {
        mtp::JudgeResult::WA
    };
    mtp::JudgeReport::new(&id, test_case_index, status, cpu_time_usage, memory_usage)
}

fn generate_special_judge_problem_report(
    id: &str,
    time_limit: u64,
    memory_limit: u64,
    problem_dir: &path::Path,
    test_case_index: usize,
    launch_result: runner::LaunchResult,
) -> mtp::JudgeReport {
    let case_dir = &problem_dir.test_case_dirs()[test_case_index];
    let input_file = case_dir.input_file();
    let output_file = case_dir.output_file();
    let answer_file = case_dir.answer_file();
    let spj_file = problem_dir.spj_file();

    let runner::LaunchResult {
        exit_code,
        real_time_usage,
        cpu_time_usage,
        memory_usage,
        tle_flag,
        mle_flag,
    } = launch_result;
    let status = if mle_flag || memory_usage >= memory_limit {
        mtp::JudgeResult::MLE
    } else if tle_flag || real_time_usage >= time::Duration::from_nanos(time_limit / 2 * 3) {
        mtp::JudgeResult::TLE
    } else if exit_code != 0 {
        mtp::JudgeResult::RE
    } else if diff::check_with_spj(&input_file, &output_file, &answer_file, &spj_file)
        .unwrap_or(false)
    {
        mtp::JudgeResult::AC
    } else {
        mtp::JudgeResult::WA
    };
    mtp::JudgeReport::new(&id, test_case_index, status, cpu_time_usage, memory_usage)
}

#[cfg(test)]
mod tests_common {
    use super::*;

    use std::fs;
    use std::path;
    use std::sync;
    use std::thread;

    use serde_json;
    use uuid::prelude::*;
    use zmq;

    use crate::communicator::EOF;

    pub const NS_PER_SEC: f64 = 1_000_000_000 as f64;
    pub const BYTES_PER_MB: f64 = (1024 * 1024) as f64;

    pub const TIME_EPS: f64 = 2.0;
    pub const MEMORY_EPS: f64 = 1.1;

    pub struct Judge {
        judge_sender: zmq::Socket,
        report_receiver: zmq::Socket,
    }

    static INIT_LOG: sync::Once = sync::Once::new();

    impl Judge {
        pub fn new(name: &str) -> Judge {
            INIT_LOG.call_once(|| env_logger::init());

            debug!("Start test judging {}", &name);
            let (judge_sender, judge_receiver) =
                create_zmq_socket_pair(&format!("inproc://{}-judge", &name));
            let (report_sender, report_receiver) =
                create_zmq_socket_pair(&format!("inproc://{}-report", &name));
            thread::spawn(move || {
                start_judging(1, judge_receiver, report_sender);
            });
            Judge {
                judge_sender,
                report_receiver,
            }
        }

        pub fn send_judge(&self, judge_task: &mtp::JudgeTask) {
            self.judge_sender
                .send(&serde_json::to_string(&judge_task).unwrap(), 0)
                .unwrap();
        }

        pub fn receive_report(&self) -> mtp::JudgeReport {
            let report_json = self.report_receiver.recv_string(0).unwrap().unwrap();
            serde_json::from_str(&report_json).unwrap()
        }
    }

    impl Drop for Judge {
        fn drop(&mut self) {
            debug!("Dropping judge task");
            self.judge_sender.send(EOF, 0).unwrap();
        }
    }

    fn create_zmq_socket_pair(endpoint: &str) -> (zmq::Socket, zmq::Socket) {
        let context = zmq::Context::new();
        let sender = context.socket(zmq::PUSH).unwrap();
        sender.connect(&endpoint).unwrap();
        let receiver = context.socket(zmq::PULL).unwrap();
        receiver.bind(&endpoint).unwrap();
        (sender, receiver)
    }

    pub fn generate_judge_task<T: AsRef<path::Path>>(
        source_file: T,
        problem_file: T,
    ) -> mtp::JudgeTask {
        let source_code = fs::read(&source_file).unwrap();
        let source_code = String::from_utf8(source_code).unwrap();
        let source = mtp::Source {
            language: String::from("cpp.gxx"),
            code: source_code,
        };
        let problem_file = fs::File::open(&problem_file).unwrap();
        let problem = serde_json::from_reader(problem_file).unwrap();
        mtp::JudgeTask {
            id: Uuid::new_v4().to_string(),
            source,
            problem,
        }
    }

    pub fn assert_report_with_limit(
        report: &mtp::JudgeReport,
        id: &str,
        status: &str,
        time: u64,
        memory: u64,
    ) {
        assert_eq!(report.id, id);
        assert_eq!(report.status, status);
        assert!(report.time <= time);
        assert!(report.memory <= memory);
    }
}

#[cfg(test)]
mod test_normal_judge {
    use super::*;
    use tests_common::*;

    const PROBLEM: &str = "example/problem.json";

    #[test]
    fn test_normal_judge_with_ac() {
        let judge_task = generate_judge_task("example/source.cpp", PROBLEM);
        let judge = Judge::new("test_normal_judge_with_ac");
        judge.send_judge(&judge_task);
        for _i in 0..judge_task.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_task.id,
                "AC",
                (1.0 * NS_PER_SEC) as u64,
                (32.0 * BYTES_PER_MB) as u64,
            );
        }
    }

    #[test]
    fn test_normal_judge_with_ce() {
        let judge_task = generate_judge_task("example/source.ce.cpp", PROBLEM);
        let judge = Judge::new("test_normal_judge_with_ce");
        judge.send_judge(&judge_task);
        let report = judge.receive_report();
        assert_report_with_limit(
            &report.into(),
            &judge_task.id,
            "CE",
            (1.0 * NS_PER_SEC) as u64,
            (32.0 * BYTES_PER_MB) as u64,
        );
    }

    #[test]
    fn test_normal_judge_with_mle() {
        let judge_task = generate_judge_task("example/source.mle.cpp", PROBLEM);
        let judge = Judge::new("test_normal_judge_with_mle");
        judge.send_judge(&judge_task);
        for _i in 0..judge_task.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_task.id,
                "MLE",
                (1.0 * NS_PER_SEC) as u64,
                (32.0 * MEMORY_EPS * BYTES_PER_MB) as u64,
            );
        }
    }

    #[test]
    fn test_normal_judge_with_re() {
        let judge_task = generate_judge_task("example/source.re.cpp", PROBLEM);
        let judge = Judge::new("test_normal_judge_with_re");
        judge.send_judge(&judge_task);
        for _i in 0..judge_task.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_task.id,
                "RE",
                (1.0 * NS_PER_SEC) as u64,
                (32.0 * BYTES_PER_MB) as u64,
            );
        }
    }

    #[test]
    fn test_normal_judge_with_tle() {
        let judge_task = generate_judge_task("example/source.tle.cpp", PROBLEM);
        let judge = Judge::new("test_normal_judge_with_tle");
        judge.send_judge(&judge_task);
        for _i in 0..judge_task.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_task.id,
                "TLE",
                (1.0 * TIME_EPS * NS_PER_SEC) as u64,
                (32.0 * BYTES_PER_MB) as u64,
            );
        }
    }

    #[test]
    fn test_normal_judge_with_wa() {
        let judge_task = generate_judge_task("example/source.wa.cpp", PROBLEM);
        let judge = Judge::new("test_normal_judge_with_wa");
        judge.send_judge(&judge_task);
        for _i in 0..judge_task.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_task.id,
                "WA",
                (1.0 * NS_PER_SEC) as u64,
                (32.0 * BYTES_PER_MB) as u64,
            );
        }
    }
}

#[cfg(test)]
mod test_spj_0 {
    use super::*;
    use tests_common::*;

    const PROBLEM: &str = "example/spj_problem_0.json";

    #[test]
    fn test_spj_0_with_ac() {
        let judge_task = generate_judge_task("example/source.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_0_with_ac");
        judge.send_judge(&judge_task);
        for _i in 0..judge_task.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_task.id,
                "AC",
                (1.0 * NS_PER_SEC) as u64,
                (32.0 * BYTES_PER_MB) as u64,
            );
        }
    }

    #[test]
    fn test_spj_0_with_ce() {
        let judge_task = generate_judge_task("example/source.ce.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_0_with_ce");
        judge.send_judge(&judge_task);
        let report = judge.receive_report();
        assert_report_with_limit(
            &report.into(),
            &judge_task.id,
            "CE",
            (1.0 * NS_PER_SEC) as u64,
            (32.0 * BYTES_PER_MB) as u64,
        );
    }

    #[test]
    fn test_spj_0_with_mle() {
        let judge_task = generate_judge_task("example/source.mle.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_0_with_mle");
        judge.send_judge(&judge_task);
        for _i in 0..judge_task.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_task.id,
                "MLE",
                (1.0 * NS_PER_SEC) as u64,
                (32.0 * MEMORY_EPS * BYTES_PER_MB) as u64,
            );
        }
    }

    #[test]
    fn test_spj_0_with_re() {
        let judge_task = generate_judge_task("example/source.re.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_0_with_re");
        judge.send_judge(&judge_task);
        for _i in 0..judge_task.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_task.id,
                "RE",
                (1.0 * NS_PER_SEC) as u64,
                (32.0 * BYTES_PER_MB) as u64,
            );
        }
    }

    #[test]
    fn test_spj_0_with_tle() {
        let judge_task = generate_judge_task("example/source.tle.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_0_with_tle");
        judge.send_judge(&judge_task);
        for _i in 0..judge_task.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_task.id,
                "TLE",
                (1.0 * TIME_EPS * NS_PER_SEC) as u64,
                (32.0 * BYTES_PER_MB) as u64,
            );
        }
    }

    #[test]
    fn test_spj_0_with_wa() {
        let judge_task = generate_judge_task("example/source.wa.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_0_with_wa");
        judge.send_judge(&judge_task);
        for _i in 0..judge_task.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_task.id,
                "WA",
                (1.0 * NS_PER_SEC) as u64,
                (32.0 * BYTES_PER_MB) as u64,
            );
        }
    }

}

#[cfg(test)]
mod test_spj_1 {
    use super::*;
    use tests_common::*;

    const PROBLEM: &str = "example/spj_problem_1.json";

    #[test]
    fn test_spj_1_with_ac() {
        let judge_task = generate_judge_task("example/source.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_1_with_ac");
        judge.send_judge(&judge_task);
        for _i in 0..judge_task.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_task.id,
                "AC",
                (1.0 * NS_PER_SEC) as u64,
                (32.0 * BYTES_PER_MB) as u64,
            );
        }
    }

    #[test]
    fn test_spj_1_with_ce() {
        let judge_task = generate_judge_task("example/source.ce.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_1_with_ce");
        judge.send_judge(&judge_task);
        let report = judge.receive_report();
        assert_report_with_limit(
            &report.into(),
            &judge_task.id,
            "CE",
            (1.0 * NS_PER_SEC) as u64,
            (32.0 * BYTES_PER_MB) as u64,
        );
    }

    #[test]
    fn test_spj_1_with_mle() {
        let judge_task = generate_judge_task("example/source.mle.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_1_with_mle");
        judge.send_judge(&judge_task);
        for _i in 0..judge_task.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_task.id,
                "MLE",
                (1.0 * NS_PER_SEC) as u64,
                (32.0 * MEMORY_EPS * BYTES_PER_MB) as u64,
            );
        }
    }

    #[test]
    fn test_spj_1_with_re() {
        let judge_task = generate_judge_task("example/source.re.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_1_with_re");
        judge.send_judge(&judge_task);
        for _i in 0..judge_task.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_task.id,
                "RE",
                (1.0 * NS_PER_SEC) as u64,
                (32.0 * BYTES_PER_MB) as u64,
            );
        }
    }

    #[test]
    fn test_spj_1_with_tle() {
        let judge_task = generate_judge_task("example/source.tle.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_1_with_tle");
        judge.send_judge(&judge_task);
        for _i in 0..judge_task.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_task.id,
                "TLE",
                (1.0 * TIME_EPS * NS_PER_SEC) as u64,
                (32.0 * BYTES_PER_MB) as u64,
            );
        }
    }

    #[test]
    fn test_spj_1_with_wa() {
        let judge_task = generate_judge_task("example/source.wa.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_1_with_wa");
        judge.send_judge(&judge_task);
        for _i in 0..judge_task.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_task.id,
                "WA",
                (1.0 * NS_PER_SEC) as u64,
                (32.0 * BYTES_PER_MB) as u64,
            );
        }
    }
}
