use std::sync::mpsc;
use std::thread::spawn;

extern crate rand;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate zmq;

mod compare;
mod compiler;
mod judge;
mod launcher;
mod mtp;

fn main() {
    let context = zmq::Context::new();
    let receiver = context
        .socket(zmq::PULL)
        .expect("Failed to create zmq PULL socket");
    receiver
        .bind("tcp://0.0.0.0:8800")
        .expect("Cannot bind to tcp://0.0.0.0:8800");
    let sender = context
        .socket(zmq::PUSH)
        .expect("Failed to create zmq PUSH socket");
    sender
        .bind("tcp://0.0.0.0:8801")
        .expect("Cannot bind to tcp://0.0.0.0:8801");

    let judge_info = receiver
        .recv_string(0)
        .expect("Failed to receive the judge information")
        .expect("Received message is not a string");
    let judge_info = mtp::JudgeInfo::from_json(&judge_info)
        .expect("Judge information is invalid. Check it at server");

    let judge_id = judge_info.id.clone();
    let mut summary_report = mtp::ReportInfo::new(
        &judge_id,
        0,
        &judge::JudgeReport::new(judge::JudgeResult::AC, 0.0, 0),
    );

    let (channel_sender, channel_receiver) = mpsc::channel::<judge::JudgeReport>();

    spawn(move || {
        judge::judge(&judge_info.source, &judge_info.problem, &channel_sender);
    });

    for (index, report) in channel_receiver.iter().enumerate() {
        sender
            .send_str(
                &mtp::ReportInfo::new(&judge_id, index, &report).to_json(),
                0,
            )
            .unwrap();
        summary_report.case_index += 1;
        if summary_report.status == "AC" {
            summary_report.status = report.status.to_str();
        }
        if report.time > summary_report.time {
            summary_report.time = report.time;
        }
        if report.memory > summary_report.memory {
            summary_report.memory = report.memory;
        }
    }
    sender.send_str(&summary_report.to_json(), 0).unwrap();
}
