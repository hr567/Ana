use std::env;
use std::sync::mpsc;
use std::thread::spawn;

#[macro_use]
extern crate serde_derive;

extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate zmq;

mod compare;
mod compiler;
mod judge;
mod launcher;
mod mtp;

use self::compiler::get_language;
use self::judge::{judge, JudgeResult};
use self::mtp::JudgeInfo;

fn main() {
    let context = zmq::Context::new();
    let socket = context.socket(&zmq::SocketType::REP);
    socket
        .bind(
            format!(
                "tcp://{}:{}",
                match env::var("ANA_ADDRESS") {
                    Ok(address) => address,
                    Err(_) => String::from("127.0.0.1"),
                },
                match env::var("ANA_PORT") {
                    Ok(port) => port,
                    Err(_) => String::from("8800"),
                }
            )
            .as_str(),
        )
        .expect("Cannot bind");

    let (language, source_code, problem) = {
        let judge_info = JudgeInfo::from_json(socket.msg_recv(0).unwrap().to_string().as_str())
            .expect("JudgeInfo is invalid");
        (
            get_language(&judge_info.language),
            judge_info.source,
            judge_info.problem,
        )
    };

    let (sender, receiver) = mpsc::sync_channel::<JudgeResult>(1);

    spawn(move || {
        judge(&language, &source_code, &problem, sender);
    });

    use self::JudgeResult::*;

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
            OLE => {
                socket
                    .msg_send(zmq::Message::from(format!("#{} OLE", i).as_str()), 0)
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
