use std::sync::{Arc, Mutex};

use futures;
use tokio_threadpool;

mod communicator;
mod compare;
mod compiler;
mod judge;
mod launcher;
mod mtp;

pub fn start_judging<T, U>(max_threads: usize, judge_receiver: &T, report_sender: &U)
where
    T: communicator::JudgeReceiver,
    U: 'static + Clone + Send + communicator::ReportSender,
{
    let pool = tokio_threadpool::Builder::new()
        .pool_size(max_threads)
        .build();
    while let Some(judge_info) = judge_receiver.receive_judge_information() {
        let report_sender = report_sender.clone();
        pool.spawn(futures::lazy(move || {
            judge::judge(&judge_info, &report_sender);
            Ok(())
        }));
    }
}

impl<T: communicator::ReportSender> communicator::ReportSender for Arc<Mutex<T>> {
    fn send_report_information(&self, report: mtp::ReportInfo) {
        self.lock()
            .expect("Failed to lock the zmq socket")
            .send_report_information(report);
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io;
    use std::path;
    use std::sync::*;
    use std::thread;

    use serde_json;
    use uuid::prelude::*;
    use zmq;

    use super::*;

    mod common {
        use super::*;

        pub const TIME_EPS: f64 = 1.0;
        pub const MEMORY_EPS: f64 = 1.0;

        pub struct Judge {
            judge_sender: zmq::Socket,
            report_receiver: zmq::Socket,
        }

        impl Judge {
            pub fn new(name: &str) -> Judge {
                let (judge_sender, judge_receiver) =
                    create_zmq_socket_pair(&format!("inproc://{}-judge", &name));
                let (report_sender, report_receiver) =
                    create_zmq_socket_pair(&format!("inproc://{}-report", &name));
                thread::spawn(move || {
                    start_judging(1, &judge_receiver, &Arc::new(Mutex::new(report_sender)));
                });
                Judge {
                    judge_sender,
                    report_receiver,
                }
            }

            pub fn send_judge_info(&self, judge_info: &mtp::JudgeInfo) {
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
            spj_source_file: Option<T>,
        ) -> io::Result<mtp::JudgeInfo> {
            let source = mtp::Source {
                language: String::from("cpp.gxx"),
                code: String::from_utf8(fs::read(&source_file)?).unwrap(),
            };
            let mut problem: mtp::Problem =
                serde_json::from_reader(fs::File::open(&problem_file)?)?;
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
            assert!(report.time <= time);
            assert!(report.memory <= memory);
        }
    }

    mod test_normal_judge {
        use super::common::*;
        use super::*;

        #[test]
        fn test_normal_judge_with_ac() -> io::Result<()> {
            let judge_info =
                generate_judge_info("example/source.cpp", "example/problem.json", None)?;
            let judge = Judge::new("test_normal_judge_with_ac");
            judge.send_judge_info(&judge_info);
            for i in 0..=judge_info.problem.len() {
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
            judge.send_judge_info(&judge_info);
            let report = judge.receive_report();
            assert_report_with_limit(&report.into(), &judge_info.id, 0, "CE", 1.0, 32.0);
            Ok(())
        }

        #[test]
        fn test_normal_judge_with_mle() -> io::Result<()> {
            let judge_info =
                generate_judge_info("example/source.mle.cpp", "example/problem.json", None)?;
            let judge = Judge::new("test_normal_judge_with_mle");
            judge.send_judge_info(&judge_info);
            for i in 0..=judge_info.problem.len() {
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
            judge.send_judge_info(&judge_info);
            for i in 0..=judge_info.problem.len() {
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
            judge.send_judge_info(&judge_info);
            for i in 0..=judge_info.problem.len() {
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
            judge.send_judge_info(&judge_info);
            for i in 0..=judge_info.problem.len() {
                let report = judge.receive_report();
                assert_report_with_limit(&report.into(), &judge_info.id, i, "WA", 1.0, 32.0);
            }
            Ok(())
        }

    }

    mod test_spj_0 {
        use super::common::*;
        use super::*;

        #[test]
        fn test_spj_0_with_ac() -> io::Result<()> {
            let judge_info = generate_judge_info(
                "example/source.cpp",
                "example/spj_problem.json",
                Some("example/spj.0.cpp"),
            )?;
            let judge = Judge::new("test_special_judge_0_with_ac");
            judge.send_judge_info(&judge_info);
            for i in 0..=judge_info.problem.len() {
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
            judge.send_judge_info(&judge_info);
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
            judge.send_judge_info(&judge_info);
            for i in 0..=judge_info.problem.len() {
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
            judge.send_judge_info(&judge_info);
            for i in 0..=judge_info.problem.len() {
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
            judge.send_judge_info(&judge_info);
            for i in 0..=judge_info.problem.len() {
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
            judge.send_judge_info(&judge_info);
            for i in 0..=judge_info.problem.len() {
                let report = judge.receive_report();
                assert_report_with_limit(&report.into(), &judge_info.id, i, "WA", 1.0, 32.0);
            }
            Ok(())
        }

    }

    mod test_spj_1 {
        use super::common::*;
        use super::*;

        #[test]
        fn test_spj_1_with_ac() -> io::Result<()> {
            let judge_info = generate_judge_info(
                "example/source.cpp",
                "example/spj_problem.json",
                Some("example/spj.1.cpp"),
            )?;
            let judge = Judge::new("test_special_judge_1_with_ac");
            judge.send_judge_info(&judge_info);
            for i in 0..=judge_info.problem.len() {
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
            judge.send_judge_info(&judge_info);
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
            judge.send_judge_info(&judge_info);
            for i in 0..=judge_info.problem.len() {
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
            judge.send_judge_info(&judge_info);
            for i in 0..=judge_info.problem.len() {
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
            judge.send_judge_info(&judge_info);
            for i in 0..=judge_info.problem.len() {
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
            judge.send_judge_info(&judge_info);
            for i in 0..=judge_info.problem.len() {
                let report = judge.receive_report();
                assert_report_with_limit(&report.into(), &judge_info.id, i, "WA", 1.0, 32.0);
            }
            Ok(())
        }
    }
}
