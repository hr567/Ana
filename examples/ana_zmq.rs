use std::result::Result;

use clap::*;
use env_logger;
use log::*;

use serde_json;
use zmq;

use ana::Ana;
use ana::{communicator, mtp};

use communicator::{Error, Receiver, Sender, EOF};

struct ZmqSocket(zmq::Socket);

impl Receiver for ZmqSocket {
    /// Receive a judge task from zmq socket.
    ///
    /// Return Err(Network) if the socket cannot receive any data from network.
    /// Return Err(Data) if received message cannot be deserialized.
    /// Return Err(Eof) if received message is EOF
    fn receive(&self) -> Result<mtp::JudgeTask, Error> {
        if let Ok(buf) = self.0.recv_bytes(0) {
            if buf == EOF.as_bytes() {
                return Err(Error::EOF);
            }
            match serde_json::from_slice(&buf) {
                Ok(res) => Ok(res),
                Err(_) => Err(Error::Data),
            }
        } else {
            Err(Error::Network)
        }
    }
}

impl Sender for ZmqSocket {
    /// Send a judge report to zmq socket.
    ///
    /// Return Err(Network) if the socket failed to send the report.
    fn send(&self, report: mtp::JudgeReport) -> Result<(), Error> {
        match self.0.send(&report.to_json(), 0) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::Network),
        }
    }
}

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
        .value_of("judge_threads")
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
    env_logger::init();

    let (judge_threads, judge_receiver_endpoint, report_sender_endpoint) = get_arguments();

    let context = zmq::Context::new();

    let judge_receiver = context.socket(zmq::PULL).unwrap();
    judge_receiver
        .bind(&judge_receiver_endpoint)
        .unwrap_or_else(|_| panic!("Failed to bind to {}", &judge_receiver_endpoint));
    debug!("Task receiver bind on {}", &judge_receiver_endpoint);

    let report_sender = context.socket(zmq::PUSH).unwrap();
    report_sender
        .bind(&report_sender_endpoint)
        .unwrap_or_else(|_| panic!("Failed to bind to {}", &report_sender_endpoint));
    debug!("Report sender bind on {}", &report_sender_endpoint);

    let judge = Ana::new(
        judge_threads,
        ZmqSocket(judge_receiver),
        ZmqSocket(report_sender),
    );

    info!("Ana start judging");
    judge.start();
    info!("Ana stop judging");
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use std::io;
    use std::path;

    use uuid::Uuid;

    const NS_PER_SEC: f64 = 1_000_000_000 as f64;
    const BYTES_PER_MB: f64 = (1024 * 1024) as f64;

    fn generate_judge_task<T: AsRef<path::Path>>(
        source_file: T,
        problem_file: T,
    ) -> mtp::JudgeTask {
        let source = mtp::Source {
            language: String::from("cpp.gxx"),
            code: String::from_utf8(fs::read(&source_file).unwrap()).unwrap(),
        };
        let problem: mtp::Problem =
            serde_json::from_reader(fs::File::open(&problem_file).unwrap()).unwrap();
        mtp::JudgeTask {
            id: Uuid::new_v4().to_string(),
            source,
            problem,
        }
    }

    #[test]
    fn test_judge_receiver() -> io::Result<()> {
        let judge_task = generate_judge_task("example/source.cpp", "example/problem.json");
        let judge_task_json = serde_json::to_string(&judge_task).unwrap();

        let context = zmq::Context::new();
        let sender = {
            let sender = context.socket(zmq::PUSH)?;
            sender.bind("inproc://test-judge-receiver")?;
            sender
        };
        let receiver = {
            let receiver = context.socket(zmq::PULL)?;
            receiver.connect("inproc://test-judge-receiver")?;
            receiver
        };

        let receiver = ZmqSocket(receiver);
        sender.send(&judge_task_json, 0).unwrap();
        assert_eq!(receiver.receive(), Ok(judge_task));
        sender.send("hello", 0).unwrap();
        assert_eq!(receiver.receive(), Err(Error::Data));

        Ok(())
    }

    #[test]
    fn test_report_sender() -> io::Result<()> {
        let report_info = mtp::JudgeReport::new(
            "test_report_sender",
            0,
            mtp::JudgeResult::AC,
            (0.8 * NS_PER_SEC) as u64,
            (13.6 * BYTES_PER_MB) as u64,
        );

        let context = zmq::Context::new();
        let sender = {
            let sender = context.socket(zmq::PUSH)?;
            sender.bind("inproc://test-report-sender")?;
            sender
        };
        let receiver = {
            let receiver = context.socket(zmq::PULL)?;
            receiver.connect("inproc://test-report-sender")?;
            receiver
        };

        ZmqSocket(sender).send(report_info.clone()).unwrap();
        assert_eq!(receiver.recv_string(0)?.unwrap(), report_info.to_json());

        Ok(())
    }
}
