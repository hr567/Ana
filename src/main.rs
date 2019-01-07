use std::env;
use std::sync::mpsc;
use std::thread;

use ana::*;

fn main() {
    set_work_dir_environment(
        &env::var("ANA_WORK_DIR")
            .unwrap_or_else(|_| String::from(env::temp_dir().to_str().unwrap())),
    );
    let (judge_receiver, report_sender) =
        get_zmq_sockets("tcp://0.0.0.0:8800", "tcp://0.0.0.0:8801");
    let (channel_sender, channel_receiver) = mpsc::channel::<JudgeReport>();
    thread::spawn(move || {
        start_reporting(&channel_receiver, &report_sender);
    });
    start_judging(&judge_receiver, &channel_sender);
}
