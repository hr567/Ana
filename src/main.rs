use std::sync;

use clap;

use ana::*;

fn main() {
    let matches = clap::App::new("Ana Judge")
        .version("0.4.2")
        .author("hr567")
        .about("A Judge for ACMers in Rust")
        .arg(
            clap::Arg::with_name("max_threads")
                .value_name("N")
                .long("max_threads")
                .short("N")
                .help("The max size of the judging thread pool")
                .env("ANA_MAX_TASKS")
                .default_value("1")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("judge_receiver_endpoint")
                .value_name("RECV_ENDPOINT")
                .long("recv_endpoint")
                .short("r")
                .help("The judge receiver binding endpoint")
                .env("ANA_RECV_ENDPOINT")
                .default_value("tcp://0.0.0.0:8800")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("report_sender_endpoint")
                .value_name("SEND_ENDPOINT")
                .long("send_endpoint")
                .short("s")
                .help("The report sender binding endpoint")
                .env("ANA_SEND_ENDPOINT")
                .default_value("tcp://0.0.0.0:8801")
                .takes_value(true),
        )
        .get_matches();

    let max_threads: usize = matches
        .value_of("max_threads")
        .unwrap()
        .parse()
        .expect("Failed to set max threads");
    let judge_receiver_endpoint = matches.value_of("judge_receive_endpoint").unwrap();
    let report_sender_endpoint = matches.value_of("report_send_endpoint").unwrap();

    let context = zmq::Context::new();
    let judge_receiver = context.socket(zmq::PULL).unwrap();
    judge_receiver
        .bind(&judge_receiver_endpoint)
        .unwrap_or_else(|_| panic!("Failed to bind to {}", &judge_receiver_endpoint));
    let report_sender = context.socket(zmq::PUSH).unwrap();
    report_sender
        .bind(&report_sender_endpoint)
        .unwrap_or_else(|_| panic!("Failed to bind to {}", &report_sender_endpoint));
    start_judging(
        max_threads,
        &judge_receiver,
        &sync::Arc::new(sync::Mutex::new(report_sender)),
    );
}
