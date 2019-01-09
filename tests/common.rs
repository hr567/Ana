use std::fs;
use std::io;
use std::path;
use std::sync::*;
use std::thread;

use serde_json;
use uuid::prelude::*;
use zmq;

use ana::mtp::*;
use ana::*;

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
            start_judging(&judge_receiver, &Arc::new(Mutex::new(report_sender)));
        });
        Judge {
            judge_sender,
            report_receiver,
        }
    }

    pub fn send_judge_info(&self, judge_info: &mtp::JudgeInfo) {
        self.judge_sender
            .send_str(&serde_json::to_string(&judge_info).unwrap(), 0)
            .unwrap();
    }

    pub fn receive_report(&self) -> ReportInfo {
        let report_json = self.report_receiver.recv_string(0).unwrap().unwrap();
        serde_json::from_str(&report_json).unwrap()
    }
}

impl Drop for Judge {
    fn drop(&mut self) {
        self.judge_sender.send_str("EOF", 0).unwrap();
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
    assert!(report.time <= time);
    assert!(report.memory <= memory);
}
