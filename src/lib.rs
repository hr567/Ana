use std::fs;
use std::path;
use std::sync;
use std::time;

mod communicator;
mod compiler;
mod diff;
mod mtp;
mod runner;

use log::*;
use tempfile;
use tokio::prelude::*;

const NS_PER_SEC: f64 = 1_000_000_000 as f64;
const BYTES_PER_MB: f64 = (1024 * 1024) as f64;

/// A simple wrapper for TempDir
/// Add some methods to get special files
#[derive(Clone)]
struct WorkDir(sync::Arc<tempfile::TempDir>);

impl WorkDir {
    pub fn new() -> WorkDir {
        let work_dir = tempfile::tempdir().expect("Failed to create a temp dir");
        debug!("Create new work directory in {:?}", work_dir.path());
        WorkDir(sync::Arc::new(work_dir))
    }

    fn join(&self, filename: &str) -> Box<path::Path> {
        self.0.path().join(filename).into_boxed_path()
    }

    pub fn source_file(&self) -> Box<path::Path> {
        self.join("source")
    }

    pub fn executable_file(&self) -> Box<path::Path> {
        self.join("main")
    }

    pub fn input_file(&self) -> Box<path::Path> {
        self.join("input")
    }

    pub fn output_file(&self) -> Box<path::Path> {
        self.join("output")
    }

    pub fn answer_file(&self) -> Box<path::Path> {
        self.join("answer")
    }

    pub fn spj_source_file(&self) -> Box<path::Path> {
        self.join("spj_source")
    }

    pub fn spj_executable_file(&self) -> Box<path::Path> {
        self.join("spj")
    }
}

/// The entry of judging
/// Generate reports for every task from receiver and send them
pub fn start_judging<T, U>(_max_threads: usize, judge_receiver: T, report_sender: U)
where
    T: Into<communicator::JudgeReceiver>,
    U: Into<communicator::ReportSender>,
{
    info!("Judging is starting");
    let judge_receiver = judge_receiver.into();
    let report_sender = report_sender.into();
    let pool = tokio_threadpool::ThreadPool::new();
    let server = judge_receiver.for_each(move |judge_info| {
        debug!("Received judge information: {:?}", &judge_info);
        let report_sender = report_sender.clone();
        let task = judge(judge_info).and_then(move |reports| {
            debug!("Generated judge reports: {:?}", &reports);
            for report in reports {
                report_sender.send_report(report);
            }
            Ok(())
        });
        tokio::spawn(task);
        Ok(())
    });
    pool.spawn(server);
    pool.shutdown_on_idle()
        .wait()
        .expect("Failed to wait thread pool to shutdown");
    info!("Judging is done");
}

fn judge(judge_info: mtp::JudgeInfo) -> impl Future<Item = Vec<mtp::ReportInfo>, Error = ()> {
    let work_dir = WorkDir::new();
    let mtp::JudgeInfo {
        id,
        source,
        problem,
    } = judge_info;
    debug!("Start compiling source code");
    compile_source(work_dir.clone(), source).and_then(move |compile_success| {
        if compile_success {
            debug!("Compile success");
            let problem_length = problem.len();
            let problem_type = problem.get_type();
            let mtp::Problem {
                time_limit,
                memory_limit,
                checker,
                test_cases,
                ..
            } = problem;
            let time_limit = (time_limit * NS_PER_SEC) as u64;
            let memory_limit = (memory_limit * BYTES_PER_MB) as u64;
            if let mtp::ProblemType::Special = problem_type {
                debug!("Building special judge");
                let build_spj_result = build_special_judge(work_dir.clone(), checker)
                    .wait()
                    .unwrap();
                assert!(build_spj_result, "Failed to build special judge");
            }
            let (report_sender, report_receiver) = sync::mpsc::channel();
            for (index, test_case) in test_cases.into_iter().enumerate() {
                debug!("Testing test case #{}", index);
                let report_sender = report_sender.clone();
                let id = id.clone();
                let work_dir = work_dir.clone();
                let mtp::TestCase { input, answer } = test_case;
                let task = launch_program(
                    work_dir.clone(),
                    id.clone(),
                    input,
                    time_limit,
                    memory_limit,
                )
                .and_then(move |launch_result| {
                    debug!("[#{}] Program is exited", index);
                    fs::write(work_dir.answer_file(), answer)
                        .expect("Failed to write to answer file");
                    let report = generate_report_from_work_dir(
                        work_dir.clone(),
                        &id,
                        index,
                        launch_result,
                        problem_type,
                        time_limit,
                        memory_limit,
                    );
                    debug!("[#{}] Generated report: {:?}", index, &report);
                    report_sender.send(report).unwrap();
                    debug!("[#{}] Report has been sent", index);
                    Ok(())
                });
                task.wait().unwrap();
            }
            let mut reports: Vec<mtp::ReportInfo> = Vec::new();
            for _ in 0..problem_length {
                let report = report_receiver.recv().unwrap();
                reports.push(report);
            }
            debug!("Collected reports: {:?}", &reports);
            Ok(reports)
        } else {
            Ok(vec![mtp::ReportInfo::new(
                &id,
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
fn generate_report_from_work_dir(
    work_dir: WorkDir,
    judge_id: &str,
    index: usize,
    launch_result: runner::LaunchResult,
    problem_type: mtp::ProblemType,
    time_limit: u64,
    memory_limit: u64,
) -> mtp::ReportInfo {
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
    } else if check_output(&work_dir, problem_type) {
        mtp::JudgeResult::AC
    } else {
        mtp::JudgeResult::WA
    };
    mtp::ReportInfo::new(
        &judge_id,
        index,
        status,
        cpu_time_usage as f64 / NS_PER_SEC,
        memory_usage as f64 / BYTES_PER_MB,
    )
}

fn compile_source(work_dir: WorkDir, source: mtp::Source) -> impl Future<Item = bool, Error = ()> {
    let source_file = work_dir.source_file();
    let executable_file = work_dir.executable_file();
    let mtp::Source { language, code } = source;
    tokio::fs::File::create(work_dir.source_file())
        .and_then(move |mut file| file.poll_write(code.as_bytes()))
        .then(move |_| compiler::compile(&language, &source_file, &executable_file))
}

fn launch_program(
    work_dir: WorkDir,
    id: String,
    input: String,
    time_limit: u64,
    memory_limit: u64,
) -> impl Future<Item = runner::LaunchResult, Error = ()> {
    let executable_file = work_dir.executable_file();
    assert!(executable_file.exists());
    let input_file = work_dir.input_file();
    let output_file = work_dir.output_file();
    tokio::fs::File::create(work_dir.input_file())
        .and_then(move |mut file| file.poll_write(input.as_bytes()))
        .then(move |_| {
            runner::launch(
                &id,
                &executable_file,
                &input_file,
                &output_file,
                time_limit,
                memory_limit,
            )
        })
}

fn build_special_judge(
    work_dir: WorkDir,
    spj_source: mtp::Source,
) -> impl Future<Item = bool, Error = ()> {
    let mtp::Source { language, code } = spj_source;
    let spj_source_file = work_dir.join("spj_source");
    let spj_executable_file = work_dir.join("spj");
    tokio::fs::File::create(work_dir.spj_source_file())
        .and_then(move |mut file| file.poll_write(code.as_bytes()))
        .then(move |_res| compiler::compile(&language, &spj_source_file, &spj_executable_file))
}

fn check_output(work_dir: &WorkDir, problem_type: mtp::ProblemType) -> bool {
    use mtp::ProblemType::*;
    match problem_type {
        Normal => diff::check(
            &work_dir.input_file(),
            &work_dir.output_file(),
            &work_dir.answer_file(),
            None,
        ),
        Special => diff::check(
            &work_dir.input_file(),
            &work_dir.output_file(),
            &work_dir.answer_file(),
            Some(&work_dir.spj_executable_file()),
        ),
    }
    .unwrap_or(false)
}

#[cfg(test)]
mod tests_common {
    use super::*;

    use std::fs;
    use std::io;
    use std::path;
    use std::sync;

    use serde_json;
    use tokio_threadpool;
    use uuid::prelude::*;
    use zmq;

    pub const TIME_EPS: f64 = 1.0;
    pub const MEMORY_EPS: f64 = 1.0;

    pub struct Judge {
        judge_sender: zmq::Socket,
        report_receiver: zmq::Socket,
        thread_pool_handler: Option<tokio_threadpool::Shutdown>,
    }

    static INIT_LOG: sync::Once = sync::Once::new();

    impl Judge {
        pub fn new(name: &str) -> Judge {
            INIT_LOG.call_once(|| env_logger::init());

            debug!("Start judging {}", &name);
            let (judge_sender, judge_receiver) =
                create_zmq_socket_pair(&format!("inproc://{}-judge", &name));
            let (report_sender, report_receiver) =
                create_zmq_socket_pair(&format!("inproc://{}-report", &name));
            let pool = tokio_threadpool::ThreadPool::new();
            pool.spawn(future::lazy(move || {
                start_judging(1, judge_receiver, report_sender);
                Ok(())
            }));
            Judge {
                judge_sender,
                report_receiver,
                thread_pool_handler: Some(pool.shutdown()),
            }
        }

        pub fn send_judge(&self, judge_info: &mtp::JudgeInfo) {
            self.judge_sender
                .send(&serde_json::to_string(&judge_info).unwrap(), 0)
                .unwrap();
        }

        pub fn receive_report(&self) -> mtp::ReportInfo {
            let report_json = self.report_receiver.recv_string(0).unwrap().unwrap();
            serde_json::from_str(&report_json).unwrap()
        }
    }

    impl Drop for Judge {
        fn drop(&mut self) {
            debug!("Dropping judge task");
            self.judge_sender.send("EOF", 0).unwrap();
            self.thread_pool_handler.take().unwrap().wait().unwrap();
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
        spj_source_file: Option<T>,
    ) -> io::Result<mtp::JudgeInfo> {
        let source = mtp::Source {
            language: String::from("cpp.gxx"),
            code: String::from_utf8(fs::read(&source_file)?).unwrap(),
        };
        let mut problem: mtp::Problem = serde_json::from_reader(fs::File::open(&problem_file)?)?;
        if let Some(spj_source_file) = spj_source_file {
            problem.checker = mtp::Source {
                language: String::from("cpp.gxx"),
                code: String::from_utf8(fs::read(&spj_source_file)?).unwrap(),
            };
        }
        Ok(mtp::JudgeInfo {
            id: Uuid::new_v4().to_string(),
            source,
            problem,
        })
    }

    pub fn assert_report_with_limit(
        report: &mtp::ReportInfo,
        id: &str,
        index: usize,
        status: &str,
        time: f64,
        memory: f64,
    ) {
        assert_eq!(report.id, id);
        assert_eq!(report.index, index);
        assert_eq!(report.status, status);
        assert!(report.time <= time * 2.0);
        assert!(report.memory <= memory);
    }
}

#[cfg(test)]
mod test_normal_judge {
    use super::*;
    use tests_common::*;

    use std::io;

    #[test]
    fn test_normal_judge_with_ac() -> io::Result<()> {
        let judge_info = generate_judge_info("example/source.cpp", "example/problem.json", None)?;
        let judge = Judge::new("test_normal_judge_with_ac");
        judge.send_judge(&judge_info);
        for i in 0..judge_info.problem.len() {
            debug!("Received the #{} report", i);
            let report = judge.receive_report();
            assert_report_with_limit(&report.into(), &judge_info.id, i, "AC", 1.0, 32.0);
        }
        Ok(())
    }

    #[test]
    fn test_normal_judge_with_ce() -> io::Result<()> {
        let judge_info =
            generate_judge_info("example/source.ce.cpp", "example/problem.json", None)?;
        let judge = Judge::new("test_normal_judge_with_ce");
        judge.send_judge(&judge_info);
        let report = judge.receive_report();
        assert_report_with_limit(&report.into(), &judge_info.id, 0, "CE", 1.0, 32.0);
        Ok(())
    }

    #[test]
    fn test_normal_judge_with_mle() -> io::Result<()> {
        let judge_info =
            generate_judge_info("example/source.mle.cpp", "example/problem.json", None)?;
        let judge = Judge::new("test_normal_judge_with_mle");
        judge.send_judge(&judge_info);
        for i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_info.id,
                i,
                "MLE",
                1.0,
                32.0 + MEMORY_EPS,
            );
        }
        Ok(())
    }

    #[test]
    fn test_normal_judge_with_re() -> io::Result<()> {
        let judge_info =
            generate_judge_info("example/source.re.cpp", "example/problem.json", None)?;
        let judge = Judge::new("test_normal_judge_with_re");
        judge.send_judge(&judge_info);
        for i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(&report.into(), &judge_info.id, i, "RE", 1.0, 32.0);
        }
        Ok(())
    }

    #[test]
    fn test_normal_judge_with_tle() -> io::Result<()> {
        let judge_info =
            generate_judge_info("example/source.tle.cpp", "example/problem.json", None)?;
        let judge = Judge::new("test_normal_judge_with_tle");
        judge.send_judge(&judge_info);
        for i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_info.id,
                i,
                "TLE",
                1.0 + TIME_EPS,
                32.0,
            );
        }
        Ok(())
    }

    #[test]
    fn test_normal_judge_with_wa() -> io::Result<()> {
        let judge_info =
            generate_judge_info("example/source.wa.cpp", "example/problem.json", None)?;
        let judge = Judge::new("test_normal_judge_with_wa");
        judge.send_judge(&judge_info);
        for i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(&report.into(), &judge_info.id, i, "WA", 1.0, 32.0);
        }
        Ok(())
    }

}

#[cfg(test)]
mod test_spj_0 {
    use super::*;
    use tests_common::*;

    use std::io;

    #[test]
    fn test_spj_0_with_ac() -> io::Result<()> {
        let judge_info = generate_judge_info(
            "example/source.cpp",
            "example/spj_problem.json",
            Some("example/spj.0.cpp"),
        )?;
        let judge = Judge::new("test_special_judge_0_with_ac");
        judge.send_judge(&judge_info);
        for i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(&report.into(), &judge_info.id, i, "AC", 1.0, 32.0);
        }
        Ok(())
    }

    #[test]
    fn test_spj_0_with_ce() -> io::Result<()> {
        let judge_info = generate_judge_info(
            "example/source.ce.cpp",
            "example/spj_problem.json",
            Some("example/spj.0.cpp"),
        )?;
        let judge = Judge::new("test_special_judge_0_with_ce");
        judge.send_judge(&judge_info);
        let report = judge.receive_report();
        assert_report_with_limit(&report.into(), &judge_info.id, 0, "CE", 1.0, 32.0);
        Ok(())
    }

    #[test]
    fn test_spj_0_with_mle() -> io::Result<()> {
        let judge_info = generate_judge_info(
            "example/source.mle.cpp",
            "example/spj_problem.json",
            Some("example/spj.0.cpp"),
        )?;
        let judge = Judge::new("test_special_judge_0_with_mle");
        judge.send_judge(&judge_info);
        for i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_info.id,
                i,
                "MLE",
                1.0,
                32.0 + MEMORY_EPS,
            );
        }
        Ok(())
    }

    #[test]
    fn test_spj_0_with_re() -> io::Result<()> {
        let judge_info = generate_judge_info(
            "example/source.re.cpp",
            "example/spj_problem.json",
            Some("example/spj.0.cpp"),
        )?;
        let judge = Judge::new("test_special_judge_0_with_re");
        judge.send_judge(&judge_info);
        for i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(&report.into(), &judge_info.id, i, "RE", 1.0, 32.0);
        }
        Ok(())
    }

    #[test]
    fn test_spj_0_with_tle() -> io::Result<()> {
        let judge_info = generate_judge_info(
            "example/source.tle.cpp",
            "example/spj_problem.json",
            Some("example/spj.0.cpp"),
        )?;
        let judge = Judge::new("test_special_judge_0_with_tle");
        judge.send_judge(&judge_info);
        for i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_info.id,
                i,
                "TLE",
                1.0 + TIME_EPS,
                32.0,
            );
        }
        Ok(())
    }

    #[test]
    fn test_spj_0_with_wa() -> io::Result<()> {
        let judge_info = generate_judge_info(
            "example/source.wa.cpp",
            "example/spj_problem.json",
            Some("example/spj.0.cpp"),
        )?;
        let judge = Judge::new("test_special_judge_0_with_wa");
        judge.send_judge(&judge_info);
        for i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(&report.into(), &judge_info.id, i, "WA", 1.0, 32.0);
        }
        Ok(())
    }

}

#[cfg(test)]
mod test_spj_1 {
    use super::*;
    use tests_common::*;

    use std::io;

    #[test]
    fn test_spj_1_with_ac() -> io::Result<()> {
        let judge_info = generate_judge_info(
            "example/source.cpp",
            "example/spj_problem.json",
            Some("example/spj.1.cpp"),
        )?;
        let judge = Judge::new("test_special_judge_1_with_ac");
        judge.send_judge(&judge_info);
        for i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(&report.into(), &judge_info.id, i, "AC", 1.0, 32.0);
        }
        Ok(())
    }

    #[test]
    fn test_spj_1_with_ce() -> io::Result<()> {
        let judge_info = generate_judge_info(
            "example/source.ce.cpp",
            "example/spj_problem.json",
            Some("example/spj.1.cpp"),
        )?;
        let judge = Judge::new("test_special_judge_1_with_ce");
        judge.send_judge(&judge_info);
        let report = judge.receive_report();
        assert_report_with_limit(&report.into(), &judge_info.id, 0, "CE", 1.0, 32.0);
        Ok(())
    }

    #[test]
    fn test_spj_1_with_mle() -> io::Result<()> {
        let judge_info = generate_judge_info(
            "example/source.mle.cpp",
            "example/spj_problem.json",
            Some("example/spj.1.cpp"),
        )?;
        let judge = Judge::new("test_special_judge_1_with_mle");
        judge.send_judge(&judge_info);
        for i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_info.id,
                i,
                "MLE",
                1.0,
                32.0 + MEMORY_EPS,
            );
        }
        Ok(())
    }

    #[test]
    fn test_spj_1_with_re() -> io::Result<()> {
        let judge_info = generate_judge_info(
            "example/source.re.cpp",
            "example/spj_problem.json",
            Some("example/spj.1.cpp"),
        )?;
        let judge = Judge::new("test_special_judge_1_with_re");
        judge.send_judge(&judge_info);
        for i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(&report.into(), &judge_info.id, i, "RE", 1.0, 32.0);
        }
        Ok(())
    }

    #[test]
    fn test_spj_1_with_tle() -> io::Result<()> {
        let judge_info = generate_judge_info(
            "example/source.tle.cpp",
            "example/spj_problem.json",
            Some("example/spj.1.cpp"),
        )?;
        let judge = Judge::new("test_special_judge_1_with_tle");
        judge.send_judge(&judge_info);
        for i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(
                &report.into(),
                &judge_info.id,
                i,
                "TLE",
                1.0 + TIME_EPS,
                32.0,
            );
        }
        Ok(())
    }

    #[test]
    fn test_spj_1_with_wa() -> io::Result<()> {
        let judge_info = generate_judge_info(
            "example/source.wa.cpp",
            "example/spj_problem.json",
            Some("example/spj.1.cpp"),
        )?;
        let judge = Judge::new("test_special_judge_1_with_wa");
        judge.send_judge(&judge_info);
        for i in 0..judge_info.problem.len() {
            let report = judge.receive_report();
            assert_report_with_limit(&report.into(), &judge_info.id, i, "WA", 1.0, 32.0);
        }
        Ok(())
    }
}
