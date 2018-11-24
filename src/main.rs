use std::env;
use std::sync::mpsc;
use std::thread::spawn;

extern crate rand;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate zmq;

mod compare;
mod compiler;
mod judge;
mod launcher;
mod mtp;

use self::judge::{judge, JudgeResult};

fn main() {
    let endpoint = format!(
        "tcp://{}:{}",
        env::var("ANA_ADDRESS").unwrap_or(String::from("0.0.0.0")),
        env::var("ANA_PORT").unwrap_or(String::from("8800"))
    );

    let context = zmq::Context::new();
    let socket = context
        .socket(zmq::REP)
        .expect("Failed to create zmq REP socket");
    socket
        .bind(endpoint.as_str())
        .expect(format!("Cannot bind to {}", endpoint).as_str());

    let judge_info = mtp::JudgeInfo::from_json(
        socket
            .recv_string(0)
            .expect("Failed to receive the judge information")
            .expect("Received message is not a string")
            .as_str(),
    )
    .expect("JudgeInfo is invalid");
    let (id, language, source_code, problem) = (
        judge_info.id,
        judge_info.language,
        judge_info.source,
        judge_info.problem,
    );

    let (sender, receiver) = mpsc::channel::<JudgeResult>();

    spawn(move || {
        judge(&language, &source_code, &problem, sender);
    });

    let mut summary_report = mtp::ReportInfo::new(id.as_str(), 0, "AC", 0.0, 0);

    for (index, res) in receiver.iter().enumerate() {
        match res {
            JudgeResult::CE => {
                socket
                    .send_str(
                        mtp::ReportInfo::new(id.as_str(), 0, "CE", 0.0, 0)
                            .to_json()
                            .as_str(),
                        0,
                    )
                    .unwrap();
                break;
            }
            JudgeResult::AC(time, memory) => {
                socket
                    .send_str(
                        mtp::ReportInfo::new(id.as_str(), index, "AC", time, memory)
                            .to_json()
                            .as_str(),
                        0,
                    )
                    .unwrap();

                if time > summary_report.time {
                    summary_report.time = time;
                }
                if memory > summary_report.memory {
                    summary_report.memory = memory;
                }
            }
            JudgeResult::WA(time, memory) => {
                socket
                    .send_str(
                        mtp::ReportInfo::new(id.as_str(), index, "WA", time, memory)
                            .to_json()
                            .as_str(),
                        0,
                    )
                    .unwrap();

                if time > summary_report.time {
                    summary_report.time = time;
                }
                if memory > summary_report.memory {
                    summary_report.memory = memory;
                }
                if summary_report.status == "AC" {
                    summary_report.status = String::from("WA");
                }
            }
            JudgeResult::TLE(time, memory) => {
                socket
                    .send_str(
                        mtp::ReportInfo::new(id.as_str(), index, "TLE", time, memory)
                            .to_json()
                            .as_str(),
                        0,
                    )
                    .unwrap();

                if time > summary_report.time {
                    summary_report.time = time;
                }
                if memory > summary_report.memory {
                    summary_report.memory = memory;
                }
                if summary_report.status == "AC" {
                    summary_report.status = String::from("TLE");
                }
            }
            JudgeResult::MLE(time, memory) => {
                socket
                    .send_str(
                        mtp::ReportInfo::new(id.as_str(), index, "MLE", time, memory)
                            .to_json()
                            .as_str(),
                        0,
                    )
                    .unwrap();

                if time > summary_report.time {
                    summary_report.time = time;
                }
                if memory > summary_report.memory {
                    summary_report.memory = memory;
                }
                if summary_report.status == "AC" {
                    summary_report.status = String::from("MLE");
                }
            }
            JudgeResult::OLE(_time, _memory) => unimplemented!("OLE flag is not support"),
            JudgeResult::RE(time, memory) => {
                socket
                    .send_str(
                        mtp::ReportInfo::new(id.as_str(), index, "RE", time, memory)
                            .to_json()
                            .as_str(),
                        0,
                    )
                    .unwrap();

                if time > summary_report.time {
                    summary_report.time = time;
                }
                if memory > summary_report.memory {
                    summary_report.memory = memory;
                }
                if summary_report.status == "AC" {
                    summary_report.status = String::from("RE");
                }
            }
        }
        socket
            .recv_bytes(0)
            .expect("Cannot receive the reply from server");
    }
    socket
        .send_str(summary_report.to_json().as_str(), 0)
        .unwrap();
}
