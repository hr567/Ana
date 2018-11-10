use std::fs::read_to_string;
use std::sync::mpsc;
use std::thread::spawn;

#[macro_use]
extern crate serde_derive;

extern crate clap;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate zmq;

pub mod compare;
pub mod compiler;
pub mod judge;
pub mod launcher;
pub mod problem;

use self::compiler::get_language;
use self::judge::{judge, JudgeResult};
use self::problem::Problem;

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

    let (sender, receiver) = mpsc::sync_channel::<JudgeResult>(1);

    spawn(move || {
        judge(&language, &source_code, &problem, sender);
    });

    use self::JudgeResult::*;

    let context = zmq::Context::new();
    let socket = context.socket(&zmq::SocketType::REQ);
    socket
        .connect("tcp://127.0.0.1:8800")
        .expect("Cannot connect to server");

    for (i, res) in receiver.iter().enumerate() {
        match res {
            CE => {
                socket
                    .msg_send(zmq::Message::from(format!("#{} CE", i).as_str()), 0)
                    .unwrap();
            }
            AC => {
                socket
                    .msg_send(zmq::Message::from(format!("#{} AC", i).as_str()), 0)
                    .unwrap();
            }
            WA => {
                socket
                    .msg_send(zmq::Message::from(format!("#{} WA", i).as_str()), 0)
                    .unwrap();
            }
            TLE => {
                socket
                    .msg_send(zmq::Message::from(format!("#{} TLE", i).as_str()), 0)
                    .unwrap();
            }
            MLE => {
                socket
                    .msg_send(zmq::Message::from(format!("#{} MLE", i).as_str()), 0)
                    .unwrap();
            }
            RE => {
                socket
                    .msg_send(zmq::Message::from(format!("#{} RE", i).as_str()), 0)
                    .unwrap();
            }
        }
        socket.recv(0, 0).unwrap();
    }
}
