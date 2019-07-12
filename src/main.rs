use std::net::IpAddr;
use std::thread::park;

use ana::start_server;
use clap::*;
use futures::Future;

fn main() {
    let (judge_threads, address, port) = get_arguments();
    let mut server = start_server(judge_threads, address, port);
    park();
    server.shutdown().wait().expect("Failed to shutdown server");
}

fn get_arguments() -> (usize, IpAddr, u16) {
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
            Arg::with_name("listen_address")
                .value_name("LISTEN_ADDRESS")
                .long("listen_address")
                .short("l")
                .help("The listening address")
                .env("ANA_ADDRESS")
                .default_value("0.0.0.0")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("listen_port")
                .value_name("LISTEN_PORT")
                .long("listen_port")
                .short("p")
                .help("The listening port")
                .env("ANA_PORT")
                .default_value("8800")
                .takes_value(true),
        )
        .get_matches();

    let judge_threads = matches
        .value_of("judge_threads")
        .unwrap()
        .parse()
        .expect("Please set judge threads currently");
    let address = matches
        .value_of("listen_address")
        .unwrap()
        .parse()
        .expect("Please set address current");
    let port = matches
        .value_of("listen_port")
        .unwrap()
        .parse()
        .expect("Please set port current");

    (judge_threads, address, port)
}
