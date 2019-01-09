use std::sync::mpsc;
use std::thread;

use ana::*;

fn main() {
    let (judge_receiver, report_sender) =
        get_zmq_sockets("tcp://0.0.0.0:8800", "tcp://0.0.0.0:8801");
    let (channel_sender, channel_receiver) = mpsc::channel::<_>();
    let reporting_task = thread::spawn(move || {
        start_reporting(&channel_receiver, &report_sender);
    });
    start_judging(&judge_receiver, &channel_sender);
    reporting_task.join().unwrap();
}
