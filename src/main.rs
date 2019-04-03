use clap::*;
use log::*;
use zmq;

use ana::start_judging;

fn get_arguments() -> (usize, String, String) {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("judge_threads")
                .value_name("N")
                .long("judge_threads")
                .short("N")
                .help("The max size of the judging thread pool")
                .env("ANA_JUDGE_THREADS")
                .default_value("1")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("judge_receiver_endpoint")
                .value_name("RECV_ENDPOINT")
                .long("recv_endpoint")
                .short("r")
                .help("The judge receiver binding endpoint")
                .env("ANA_RECV_ENDPOINT")
                .default_value("tcp://0.0.0.0:8800")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("report_sender_endpoint")
                .value_name("SEND_ENDPOINT")
                .long("send_endpoint")
                .short("t")
                .help("The report sender binding endpoint")
                .env("ANA_SEND_ENDPOINT")
                .default_value("tcp://0.0.0.0:8801")
                .takes_value(true),
        )
        .get_matches();

    let judge_threads: usize = matches
        .value_of("max_judge_threads")
        .unwrap()
        .parse()
        .expect("Please set environment or arguments current");
    let judge_receiver_endpoint = matches.value_of("judge_receiver_endpoint").unwrap();
    let report_sender_endpoint = matches.value_of("report_sender_endpoint").unwrap();

    (
        judge_threads,
        judge_receiver_endpoint.to_owned(),
        report_sender_endpoint.to_owned(),
    )
}

fn main() {
    let (judge_threads, judge_receiver_endpoint, report_sender_endpoint) = get_arguments();

    let context = zmq::Context::new();

    let judge_receiver = context.socket(zmq::PULL).unwrap();
    judge_receiver
        .bind(&judge_receiver_endpoint)
        .unwrap_or_else(|_| panic!("Failed to bind to {}", &judge_receiver_endpoint));
    debug!("Judge receiver bind on {}", &judge_receiver_endpoint);

    let report_sender = context.socket(zmq::PUSH).unwrap();
    report_sender
        .bind(&report_sender_endpoint)
        .unwrap_or_else(|_| panic!("Failed to bind to {}", &report_sender_endpoint));
    debug!("Report sender bind on {}", &report_sender_endpoint);

    info!("Ana start judging");
    start_judging(judge_threads, judge_receiver, report_sender);
}
