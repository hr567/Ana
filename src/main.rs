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

use self::judge::{judge, JudgeResult};

fn main() {
    let context = zmq::Context::new();
    let socket = context.socket(&zmq::SocketType::REP);
    socket
        .bind(
            format!(
                "tcp://{}:{}",
                env::var("ANA_ADDRESS").unwrap_or(String::from("0.0.0.0")),
                env::var("ANA_PORT").unwrap_or(String::from("8800"))
            )
            .as_str(),
        )
        .expect("Cannot bind");

    let judge_info = mtp::JudgeInfo::from_json(socket.msg_recv(0).unwrap().to_string().as_str())
        .expect("JudgeInfo is invalid");
    let (language, source_code, problem) =
        (judge_info.language, judge_info.source, judge_info.problem);

    let (sender, receiver) = mpsc::channel::<JudgeResult>();

    spawn(move || {
        judge(&language, &source_code, &problem, sender);
    });

    use self::JudgeResult::*;

    for res in receiver {
        match res {
            CE => {
                socket
                    .msg_send(
                        zmq::Message::from(mtp::ReportInfo::new("CE", 0.0, 0).to_json().as_str()),
                        0,
                    )
                    .unwrap();
            }
            AC(time, memory) => {
                socket
                    .msg_send(
                        zmq::Message::from(
                            mtp::ReportInfo::new("AC", time, memory).to_json().as_str(),
                        ),
                        0,
                    )
                    .unwrap();
            }
            WA(time, memory) => {
                socket
                    .msg_send(
                        zmq::Message::from(
                            mtp::ReportInfo::new("WA", time, memory).to_json().as_str(),
                        ),
                        0,
                    )
                    .unwrap();
            }
            TLE(time, memory) => {
                socket
                    .msg_send(
                        zmq::Message::from(
                            mtp::ReportInfo::new("TLE", time, memory).to_json().as_str(),
                        ),
                        0,
                    )
                    .unwrap();
            }
            MLE(time, memory) => {
                socket
                    .msg_send(
                        zmq::Message::from(
                            mtp::ReportInfo::new("MLE", time, memory).to_json().as_str(),
                        ),
                        0,
                    )
                    .unwrap();
            }
            OLE(_time, _memory) => unimplemented!("OLE flag is not support"),
            RE(time, memory) => {
                socket
                    .msg_send(
                        zmq::Message::from(
                            mtp::ReportInfo::new("RE", time, memory).to_json().as_str(),
                        ),
                        0,
                    )
                    .unwrap();
            }
        }
        socket.recv(0, 0).unwrap();
    }
}
