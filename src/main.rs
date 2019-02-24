use clap;
use log::*;
use zmq;

use ana::start_judging;

fn get_arguments() -> (usize, usize, String, String) {
    let matches = clap::App::new("Ana Judge")
        .version("0.5.0")
        .author("hr567")
        .about("A Judge for ACMers in Rust")
        .arg(
            clap::Arg::with_name("max_compile_threads")
                .value_name("M")
                .long("max_compile_threads")
                .short("M")
                .help("The max size of the compiling thread pool (not support now)")
                .env("ANA_MAX_COMPILE_TASKS")
                .default_value("1")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("max_judge_threads")
                .value_name("N")
                .long("max_judge_threads")
                .short("N")
                .help("The max size of the judging thread pool (not support now)")
                .env("ANA_MAX_JUDGE_TASKS")
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

    let max_compile_threads: usize = matches
        .value_of("max_compile_threads")
        .unwrap()
        .parse()
        .expect("Please set environment or arguments current");
    let max_judge_threads: usize = matches
        .value_of("max_judge_threads")
        .unwrap()
        .parse()
        .expect("Please set environment or arguments current");
    let judge_receiver_endpoint = matches.value_of("judge_receive_endpoint").unwrap();
    let report_sender_endpoint = matches.value_of("report_send_endpoint").unwrap();

    (
        max_compile_threads,
        max_judge_threads,
        judge_receiver_endpoint.to_owned(),
        report_sender_endpoint.to_owned(),
    )
}

fn main() {
    let (max_compile_threads, max_judge_threads, judge_receiver_endpoint, report_sender_endpoint) =
        get_arguments();
    let context = zmq::Context::new();
    let judge_receiver = context.socket(zmq::PULL).unwrap();
    judge_receiver
        .bind(&judge_receiver_endpoint)
        .unwrap_or_else(|_| panic!("Failed to bind to {}", &judge_receiver_endpoint));
    info!("Bind receiver on {}", &judge_receiver_endpoint);
    let report_sender = context.socket(zmq::PUSH).unwrap();
    report_sender
        .bind(&report_sender_endpoint)
        .unwrap_or_else(|_| panic!("Failed to bind to {}", &report_sender_endpoint));
    info!("Bind sender on {}", &report_sender_endpoint);
    start_judging(
        max_compile_threads,
        max_judge_threads,
        judge_receiver,
        report_sender,
    );
}
