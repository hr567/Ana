use std::env;
use std::sync;

use ana::*;

fn main() {
    let judge_receive_endpoint =
        env::var("ANA_RECV_ENDPOINT").unwrap_or_else(|_| String::from("tcp://0.0.0.0:8800"));
    let report_send_endpoint =
        env::var("ANA_SEND_ENDPOINT").unwrap_or_else(|_| String::from("tcp://0.0.0.0:8801"));

    let context = zmq::Context::new();
    let judge_receiver = context.socket(zmq::PULL).unwrap();
    judge_receiver
        .bind(&judge_receive_endpoint)
        .unwrap_or_else(|_| panic!("Failed to bind to {}", &judge_receive_endpoint));
    let report_sender = context.socket(zmq::PUSH).unwrap();
    report_sender
        .bind(&report_send_endpoint)
        .unwrap_or_else(|_| panic!("Failed to bind to {}", &report_send_endpoint));
    start_judging(
        &judge_receiver,
        &sync::Arc::new(sync::Mutex::new(report_sender)),
    );
}
