use ana::*;

use std::fs;
use std::path;
use std::sync;
use std::thread;

use serde_json;
use uuid::prelude::*;
use zmq;

use communicator::EOF;

pub const NS_PER_SEC: f64 = 1_000_000_000 as f64;
pub const BYTES_PER_MB: f64 = (1024 * 1024) as f64;

pub const TIME_EPS: f64 = 2.0;
pub const MEMORY_EPS: f64 = 1.1;

pub const SOURCE_AC: &str = "example/source.cpp";
pub const SOURCE_CE: &str = "example/source.ce.cpp";
pub const SOURCE_MLE: &str = "example/source.mle.cpp";
pub const SOURCE_RE: &str = "example/source.re.cpp";
pub const SOURCE_TLE: &str = "example/source.tle.cpp";
pub const SOURCE_WA: &str = "example/source.wa.cpp";

pub struct Judge {
    judge_sender: zmq::Socket,
    report_receiver: zmq::Socket,
}

static INIT_LOG: sync::Once = sync::Once::new();

impl Judge {
    pub fn new(name: &str) -> Judge {
        INIT_LOG.call_once(|| env_logger::init());

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
