use std::sync::mpsc;
use std::thread;

use serde_json;

mod compare;
mod compiler;
mod judge;
mod launcher;
pub mod mtp;

pub use self::judge::{JudgeReport, JudgeResult};

pub fn get_zmq_sockets(recv_endpoint: &str, send_endpoint: &str) -> (zmq::Socket, zmq::Socket) {
    let context = zmq::Context::new();
    let receiver = context.socket(zmq::PULL).unwrap();
    receiver
        .bind(&recv_endpoint)
        .unwrap_or_else(|_| panic!("Failed to bind to {}", &recv_endpoint));
    let sender = context.socket(zmq::PUSH).unwrap();
    sender
        .bind(&send_endpoint)
        .unwrap_or_else(|_| panic!("Failed to bind to {}", &send_endpoint));
    (receiver, sender)
}

pub fn start_reporting(
    judge_report_receiver: &mpsc::Receiver<judge::JudgeReport>,
    report_sender: &zmq::Socket,
) {
    while let Ok(report) = judge_report_receiver.recv() {
        let report: mtp::ReportInfo = report.into();
        report_sender
            .send_str(&report.to_json(), 0)
            .expect("Failed to send the report information");
    }
}

pub fn start_judging(
    judge_receiver: &zmq::Socket,
    judge_report_sender: &mpsc::Sender<judge::JudgeReport>,
) {
    loop {
        let judge_info = judge_receiver
            .recv_string(0)
            .expect("Failed to receive the judge information")
            .expect("Received message is not a string");
        if judge_info == "EOF" {
            return;
        }
        let judge_info: mtp::JudgeInfo = serde_json::from_str(&judge_info)
            .expect("Judge information is invalid. Check it at server");

        let judge_report_sender = judge_report_sender.clone();
        thread::spawn(move || {
            judge::judge(&judge_info, &judge_report_sender);
        });
    }
}
