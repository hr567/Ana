use std::sync::{Arc, Mutex};

use futures;
use serde_json;
use tokio_threadpool;
use zmq;

mod communicator;
mod compare;
mod compiler;
mod judge;
mod launcher;
pub mod mtp;

pub fn start_judging<
    T: communicator::JudgeReceiver,
    U: 'static + Clone + Send + communicator::ReportSender,
>(
    max_threads: usize,
    judge_receiver: &T,
    report_sender: &U,
) {
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

impl communicator::JudgeReceiver for zmq::Socket {
    fn receive_judge_information(&self) -> Option<mtp::JudgeInfo> {
        let judge_info = self
            .recv_string(0)
            .expect("Failed to receive the judge information")
            .expect("Received message is not a string");
        if let Ok(judge_info) = serde_json::from_str(&judge_info) {
            Some(judge_info)
        } else {
            None
        }
    }
}

impl communicator::ReportSender for zmq::Socket {
    fn send_report_information(&self, report: mtp::ReportInfo) {
        self.send_str(&report.to_json(), 0)
            .expect("Failed to send the report information");
    }
}

impl<T: communicator::ReportSender> communicator::ReportSender for Arc<Mutex<T>> {
    fn send_report_information(&self, report: mtp::ReportInfo) {
        self.lock()
            .expect("Failed to lock the zmq socket")
            .send_report_information(report);
    }
}
