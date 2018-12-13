use std::env;
use std::sync::mpsc;
use std::thread::spawn;

use ana::*;

const US_PER_SEC: f64 = (1000 * 1000) as f64;
const BYTES_PER_MB: f64 = (1024 * 1024) as f64;

fn main() {
    env::set_var(
        "ANA_WORK_DIR",
        env::var("ANA_WORK_DIR")
            .unwrap_or_else(|_| String::from(env::temp_dir().to_str().unwrap())),
    );

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

    env::set_var("ANA_JUDGE_ID", &judge_info.id);
    let mut summary_report =
        mtp::ReportInfo::new(0, &judge::JudgeReport::new(judge::JudgeResult::AC, 0, 0));

    let (channel_sender, channel_receiver) = mpsc::channel::<judge::JudgeReport>();

    spawn(move || {
        judge::judge(&judge_info, &channel_sender);
    });

    for (index, report) in channel_receiver.iter().enumerate() {
        sender
            .send_str(&mtp::ReportInfo::new(index, &report).to_json(), 0)
            .unwrap();

        summary_report.case_index += 1;
        if summary_report.status == "AC" {
            summary_report.status = report.status.to_string();
        }
        if report.time as f64 / US_PER_SEC > summary_report.time {
            summary_report.time = report.time as f64 / US_PER_SEC;
        }
        if report.memory as f64 / BYTES_PER_MB > summary_report.memory {
            summary_report.memory = report.memory as f64 / BYTES_PER_MB;
        }
    }
    sender.send_str(&summary_report.to_json(), 0).unwrap();
}
