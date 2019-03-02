use std::sync;
use std::time;

use log::*;
use tokio::prelude::*;

mod communicator;
mod compiler;
mod diff;
mod mtp;
mod runner;
mod workspace;

use communicator::*;
use workspace::*;

const NS_PER_SEC: f64 = 1_000_000_000 as f64;
const BYTES_PER_MB: f64 = (1024 * 1024) as f64;

/// The entry of judging
/// Generate reports for every task from receiver and send them
pub fn start_judging<T, U>(
    _max_compile_threads: usize,
    _max_judge_threads: usize,
    judge_receiver: impl Into<TaskReceiver<T>>,
    report_sender: impl Into<ReportSender<U>>,
) where
    T: Receiver + Send + 'static,
    U: Sender + Send + 'static,
{
    let judge_receiver = judge_receiver.into();
    let report_sender = report_sender.into();
    let server = judge_receiver.for_each(move |judge_task| {
        debug!("Received judge information: {:?}", &judge_task);
        let report_sender = report_sender.clone();
        let task = judge(judge_task).and_then(move |reports| {
            debug!("Generated judge reports: {:?}", &reports);
            for report in reports {
                report_sender
                    .send(report)
                    .unwrap_or_else(|_| error!("Failed to send report"));
            }
            Ok(())
        });
        tokio::spawn(task);
        Ok(())
    });
    tokio::run(server);
}

fn judge(judge_task: mtp::JudgeTask) -> impl Future<Item = Vec<mtp::JudgeReport>, Error = ()> {
    debug!("Start judging task :{}", &judge_task.id);
    let work_dir = WorkSpace::new();
    work_dir.prepare_judge_task(judge_task);
    debug!("Start compiling source code");
    generate_compile_future(work_dir.clone()).and_then(move |compile_success| {
        if compile_success {
            debug!("Compile success");

            work_dir.remount_runtime_dir();
            if work_dir.problem_dir().spj_path().exists() {
                assert!(
                    build_special_judge(work_dir.clone()).expect("Failed to build special judge"),
                    "Failed to build special judge"
                );
            }

            let (report_sender, report_receiver) = sync::mpsc::channel();
            for (index, task) in generate_launch_future(work_dir.clone())
                .into_iter()
                .enumerate()
            {
                debug!("Testing test case #{}", index);
                // Clone these values for the move lambda
                let report_sender = report_sender.clone();
                let work_dir = work_dir.clone();

                let task = task.and_then(move |launch_result| {
                    debug!("[#{}] Program is exited", index);
                    let report = generate_report(work_dir, index, launch_result);
                    debug!("[#{}] Generated report: {:?}", index, &report);
                    report_sender.send(report).unwrap();
                    Ok(())
                });
                tokio::spawn(task);
            }
            drop(report_sender);
            let reports: Vec<mtp::JudgeReport> = report_receiver.iter().collect();
            debug!("Collected reports: {:?}", &reports);
            Ok(reports)
        } else {
            Ok(vec![mtp::JudgeReport::new(
                &work_dir.get_id(),
                0,
                mtp::JudgeResult::CE,
                0.0,
                0.0,
            )])
        }
    })
}

/// Generate the judge report using given data.
/// The order of different cases is important
/// because TLE or RE may be caused by MLE.
/// So check memory usage first.
/// If the cpu time usage being much smaller than
/// real time usage means that there are too many
/// threads working in one time or the program use sleep.
fn generate_report(
    work_dir: WorkSpace,
    index: usize,
    launch_result: runner::LaunchResult,
) -> mtp::JudgeReport {
    let id = work_dir.get_id();
    let time_limit = work_dir.problem_dir().get_time_limit();
    let memory_limit = work_dir.problem_dir().get_memory_limit();
    let case_dir = work_dir.problem_dir().get_test_case_path(index).unwrap();
    let input_file = case_dir.input_file();
    let output_file = case_dir.output_file();
    let answer_file = case_dir.answer_file();
    let spj_file = work_dir.problem_dir().spj_path().executable_file();

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
    } else if tle_flag || real_time_usage >= time::Duration::from_nanos(time_limit * 10) {
        mtp::JudgeResult::TLE
    } else if exit_code != 0 {
        mtp::JudgeResult::RE
    } else if diff::check(
        &input_file,
        &output_file,
        &answer_file,
        if spj_file.exists() {
            Some(spj_file.as_ref())
        } else {
            None
        },
    )
    .unwrap_or(false)
    {
        mtp::JudgeResult::AC
    } else {
        mtp::JudgeResult::WA
    };
    mtp::JudgeReport::new(
        &id,
        index,
        status,
        cpu_time_usage as f64 / NS_PER_SEC,
        memory_usage as f64 / BYTES_PER_MB,
    )
}

fn generate_compile_future(work_dir: WorkSpace) -> impl Future<Item = bool, Error = ()> {
    let language = work_dir.source_dir().get_language();
    let source_file = work_dir.source_dir().source_file();
    let executable_file = work_dir.runtime_dir().executable_file();
    compiler::compile(&language, &source_file, &executable_file)
}

fn generate_launch_future(
    work_dir: WorkSpace,
) -> Vec<impl Future<Item = runner::LaunchResult, Error = ()>> {
    let mut res = Vec::new();
    let time_limit = work_dir.problem_dir().get_time_limit();
    let memory_limit = work_dir.problem_dir().get_memory_limit();

    for test_case in work_dir.problem_dir().test_cases() {
        let runtime_dir = work_dir.runtime_dir();
        let input_file = test_case.input_file();
        let output_file = test_case.output_file();

        let task = future::lazy(move || {
            runner::launch(
                &runtime_dir,
                &input_file,
                &output_file,
                time_limit,
                memory_limit,
            )
        });
        res.push(task);
    }
    res
}

fn build_special_judge(work_dir: WorkSpace) -> Result<bool, ()> {
    let language = work_dir.spj_path().get_language();
    let spj_source_file = work_dir.spj_path().source_file();
    let spj_executable_file = work_dir.spj_path().executable_file();
    compiler::compile(&language, &spj_source_file, &spj_executable_file).wait()
}

#[cfg(test)]
mod tests_common {
    use super::*;

    use std::fs;
    use std::path;
    use std::thread;

    use serde_json;
    use uuid::prelude::*;
    use zmq;

    pub const TIME_EPS: f64 = 1.0;
    pub const MEMORY_EPS: f64 = 1.0;

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
                start_judging(1, 1, judge_receiver, report_sender);
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
            self.judge_sender.send("EOF", 0).unwrap();
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

    pub fn generate_judge_info<T: AsRef<path::Path>>(
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
        time: f64,
        memory: f64,
    ) {
        assert_eq!(report.id, id);
        assert_eq!(report.status, status);
        assert!(report.time <= time * 2.0);
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
        let judge_info = generate_judge_info("example/source.cpp", PROBLEM);
        let judge = Judge::new("test_normal_judge_with_ac");
        judge.send_judge(&judge_info);
        for _i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(&report.into(), &judge_info.id, "AC", 1.0, 32.0);
        }
    }

    #[test]
    fn test_normal_judge_with_ce() {
        let judge_info = generate_judge_info("example/source.ce.cpp", PROBLEM);
        let judge = Judge::new("test_normal_judge_with_ce");
        judge.send_judge(&judge_info);
        let report = judge.receive_report();
        assert_report_with_limit(&report.into(), &judge_info.id, "CE", 1.0, 32.0);
    }

    #[test]
    fn test_normal_judge_with_mle() {
        let judge_info = generate_judge_info("example/source.mle.cpp", PROBLEM);
        let judge = Judge::new("test_normal_judge_with_mle");
        judge.send_judge(&judge_info);
        for _i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_info.id,
                "MLE",
                1.0,
                32.0 + MEMORY_EPS,
            );
        }
    }

    #[test]
    fn test_normal_judge_with_re() {
        let judge_info = generate_judge_info("example/source.re.cpp", PROBLEM);
        let judge = Judge::new("test_normal_judge_with_re");
        judge.send_judge(&judge_info);
        for _i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(&report.into(), &judge_info.id, "RE", 1.0, 32.0);
        }
    }

    #[test]
    fn test_normal_judge_with_tle() {
        let judge_info = generate_judge_info("example/source.tle.cpp", PROBLEM);
        let judge = Judge::new("test_normal_judge_with_tle");
        judge.send_judge(&judge_info);
        for _i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(&report.into(), &judge_info.id, "TLE", 1.0 + TIME_EPS, 32.0);
        }
    }

    #[test]
    fn test_normal_judge_with_wa() {
        let judge_info = generate_judge_info("example/source.wa.cpp", PROBLEM);
        let judge = Judge::new("test_normal_judge_with_wa");
        judge.send_judge(&judge_info);
        for _i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(&report.into(), &judge_info.id, "WA", 1.0, 32.0);
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
        let judge_info = generate_judge_info("example/source.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_0_with_ac");
        judge.send_judge(&judge_info);
        for _i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(&report.into(), &judge_info.id, "AC", 1.0, 32.0);
        }
    }

    #[test]
    fn test_spj_0_with_ce() {
        let judge_info = generate_judge_info("example/source.ce.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_0_with_ce");
        judge.send_judge(&judge_info);
        let report = judge.receive_report();
        assert_report_with_limit(&report.into(), &judge_info.id, "CE", 1.0, 32.0);
    }

    #[test]
    fn test_spj_0_with_mle() {
        let judge_info = generate_judge_info("example/source.mle.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_0_with_mle");
        judge.send_judge(&judge_info);
        for _i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_info.id,
                "MLE",
                1.0,
                32.0 + MEMORY_EPS,
            );
        }
    }

    #[test]
    fn test_spj_0_with_re() {
        let judge_info = generate_judge_info("example/source.re.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_0_with_re");
        judge.send_judge(&judge_info);
        for _i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(&report.into(), &judge_info.id, "RE", 1.0, 32.0);
        }
    }

    #[test]
    fn test_spj_0_with_tle() {
        let judge_info = generate_judge_info("example/source.tle.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_0_with_tle");
        judge.send_judge(&judge_info);
        for _i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(&report.into(), &judge_info.id, "TLE", 1.0 + TIME_EPS, 32.0);
        }
    }

    #[test]
    fn test_spj_0_with_wa() {
        let judge_info = generate_judge_info("example/source.wa.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_0_with_wa");
        judge.send_judge(&judge_info);
        for _i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(&report.into(), &judge_info.id, "WA", 1.0, 32.0);
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
        let judge_info = generate_judge_info("example/source.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_1_with_ac");
        judge.send_judge(&judge_info);
        for _i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(&report.into(), &judge_info.id, "AC", 1.0, 32.0);
        }
    }

    #[test]
    fn test_spj_1_with_ce() {
        let judge_info = generate_judge_info("example/source.ce.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_1_with_ce");
        judge.send_judge(&judge_info);
        let report = judge.receive_report();
        assert_report_with_limit(&report.into(), &judge_info.id, "CE", 1.0, 32.0);
    }

    #[test]
    fn test_spj_1_with_mle() {
        let judge_info = generate_judge_info("example/source.mle.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_1_with_mle");
        judge.send_judge(&judge_info);
        for _i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_info.id,
                "MLE",
                1.0,
                32.0 + MEMORY_EPS,
            );
        }
    }

    #[test]
    fn test_spj_1_with_re() {
        let judge_info = generate_judge_info("example/source.re.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_1_with_re");
        judge.send_judge(&judge_info);
        for _i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(&report.into(), &judge_info.id, "RE", 1.0, 32.0);
        }
    }

    #[test]
    fn test_spj_1_with_tle() {
        let judge_info = generate_judge_info("example/source.tle.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_1_with_tle");
        judge.send_judge(&judge_info);
        for _i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(&report.into(), &judge_info.id, "TLE", 1.0 + TIME_EPS, 32.0);
        }
    }

    #[test]
    fn test_spj_1_with_wa() {
        let judge_info = generate_judge_info("example/source.wa.cpp", PROBLEM);
        let judge = Judge::new("test_special_judge_1_with_wa");
        judge.send_judge(&judge_info);
        for _i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(&report.into(), &judge_info.id, "WA", 1.0, 32.0);
        }
    }
}
