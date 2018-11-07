use std::fs::read_to_string;
use std::sync::mpsc;
use std::thread::spawn;

extern crate Ana;
extern crate clap;
extern crate zmq;

use Ana::{
    compiler::get_language,
    judge::{judge, JudgeResult},
    problem::Problem,
};

fn main() {
    let cli_matches = clap::App::new("Ana Judge")
        .version("0.0.1")
        .author("hr567 <hr567@hr567.me>")
        .about("A judge for ACM")
        .arg(
            clap::Arg::with_name("language")
                .short("l")
                .long("language")
                .value_name("language")
                .required(true)
                .help("The language of the source file")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("source")
                .short("i")
                .long("source")
                .value_name("source_file")
                .required(true)
                .help("The path of the source file")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("problem")
                .short("p")
                .long("problem")
                .value_name("problem_file")
                .required(true)
                .help("The path of the problem file")
                .takes_value(true),
        )
        .get_matches();

    let language = cli_matches.value_of("language").unwrap();
    let source_file = cli_matches.value_of("source").unwrap();
    let problem_file = cli_matches.value_of("problem").unwrap();

    let language = get_language(language);
    let source_code = read_to_string(source_file).expect("Cannot read the source file");
    let problem = {
        let problem_json = read_to_string(problem_file).expect("Cannot read the problem file");
        Problem::from_json(problem_json.as_str()).expect("The problem is invalid")
    };

    let (sender, receiver) = mpsc::channel();

    spawn(move || {
        use self::JudgeResult::*;

        let context = zmq::Context::new();
        let socket = context.socket(&zmq::SocketType::REQ);
        socket.connect("tcp://localhost:8800").unwrap();

        for _ in 0.. {
            match receiver.recv() {
                Ok(result) => match result {
                    CE => {
                        socket.msg_send(zmq::Message::new(), 0).unwrap();
                    }
                    AC => {
                        socket.msg_send(zmq::Message::new(), 0).unwrap();
                    }
                    WA => {
                        socket.msg_send(zmq::Message::new(), 0).unwrap();
                    }
                    TLE => {
                        socket.msg_send(zmq::Message::new(), 0).unwrap();
                    }
                    MLE => {
                        socket.msg_send(zmq::Message::new(), 0).unwrap();
                    }
                    RE => {
                        socket.msg_send(zmq::Message::new(), 0).unwrap();
                    }
                },
                Err(_) => {
                    break;
                }
            }
        }
    });

    judge(&language, &source_code, &problem, sender);

    println!("Hello, world!");
}
