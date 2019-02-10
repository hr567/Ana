use crate::mtp;
use serde_json;
use zmq;

use super::{Error, Receiver, Sender};

impl Receiver for zmq::Socket {
    /// Receive a judge task from zmq socket.
    ///
    /// Return Err(Network) if the socket cannot receive any data from network.
    /// Return Err(Data) if received message cannot be deserialized.
    fn receive(&self) -> Result<mtp::JudgeTask, Error> {
        if let Ok(buf) = self.recv_bytes(0) {
            match serde_json::from_slice(&buf) {
                Ok(res) => Ok(res),
                Err(_) => Err(Error::Data),
            }
        } else {
            Err(Error::Network)
        }
    }
}

impl Sender for zmq::Socket {
    /// Send a judge report to zmq socket.
    ///
    /// Return Err(Network) if the socket failed to send the report.
    fn send(&self, report: mtp::JudgeReport) -> Result<(), Error> {
        match self.send(&report.to_json(), 0) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::Network),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use std::io;
    use std::path;

    use uuid::prelude::*;

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

        sender.send(&judge_task_json, 0).unwrap();
        assert_eq!(receiver.receive(), Ok(judge_task));
        sender.send("hello", 0).unwrap();
        assert_eq!(receiver.receive(), Err(Error::Data));

        Ok(())
    }

    #[test]
    fn test_report_sender() -> io::Result<()> {
        let report_info =
            mtp::JudgeReport::new("test_report_sender", 0, mtp::JudgeResult::AC, 0.8, 13.6);

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

        Sender::send(&sender, report_info.clone()).unwrap();
        assert_eq!(receiver.recv_string(0)?.unwrap(), report_info.to_json());

        Ok(())
    }
}
