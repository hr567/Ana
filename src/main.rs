use std::sync;

use ana::*;

fn get_zmq_sockets(recv_endpoint: &str, send_endpoint: &str) -> (zmq::Socket, zmq::Socket) {
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

fn main() {
    let (judge_receiver, report_sender) =
        get_zmq_sockets("tcp://0.0.0.0:8800", "tcp://0.0.0.0:8801");
    start_judging(
        judge_receiver,
        sync::Arc::new(sync::Mutex::new(report_sender)),
    );
}
