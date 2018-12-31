use std::env;
use std::sync::mpsc;
use std::thread;

pub use ana_compiler as compiler;
pub use ana_launcher as launcher;
pub use ana_mtp as mtp;

pub mod compare;
pub mod judge;

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

    let (channel_sender, channel_receiver) = mpsc::channel::<judge::JudgeReport>();

    thread::spawn(move || loop {
        let report: mtp::ReportInfo = channel_receiver.recv().unwrap().into();
        sender
            .send_str(&report.to_json(), 0)
            .expect("Failed to send the report information");
    });

    loop {
        let judge_info = receiver
            .recv_string(0)
            .expect("Failed to receive the judge information")
            .expect("Received message is not a string");
        let judge_info = mtp::JudgeInfo::from_json(&judge_info)
            .expect("Judge information is invalid. Check it at server");

        let channel_sender = channel_sender.clone();
        thread::Builder::new()
            .name(judge_info.id.clone())
            .spawn(move || {
                judge::judge(&judge_info, &channel_sender);
            })
            .unwrap();
    }
}
