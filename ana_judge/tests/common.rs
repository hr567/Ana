use std::fs;
use std::io;
use std::path;
use std::process;

use serde_json;
use uuid::prelude::*;
use zmq;

use ana_mtp as mtp;

pub struct Container {
    id: String,
}

impl Container {
    pub fn new() -> io::Result<Container> {
        let id = process::Command::new("docker")
            .arg("container")
            .arg("run")
            .arg("-e=RUST_BACKTRACE=1")
            .arg("--privileged")
            .arg("-d")
            .arg("hr567/ana")
            .stdout(process::Stdio::piped())
            .output()?
            .stdout;
        Ok(Container {
            id: String::from_utf8(id).unwrap().trim().to_string(),
        })
    }

    pub fn ip_address(&self) -> io::Result<String> {
        let configs = process::Command::new("docker")
            .arg("container")
            .arg("inspect")
            .arg(&self.id)
            .stdout(process::Stdio::piped())
            .output()?
            .stdout;
        let configs: serde_json::Value = serde_json::from_slice(&configs).unwrap();
        Ok(String::from(
            configs[0]["NetworkSettings"]["IPAddress"].as_str().unwrap(),
        ))
    }
}

impl Drop for Container {
    fn drop(&mut self) {
        process::Command::new("docker")
            .arg("container")
            .arg("wait")
            .arg(&self.id)
            .stdout(process::Stdio::null())
            .spawn()
            .expect("Failed to wait a Ana container exit")
            .wait()
            .unwrap();
        process::Command::new("docker")
            .arg("container")
            .arg("rm")
            .arg(&self.id)
            .stdout(process::Stdio::null())
            .spawn()
            .expect("Failed to remove a Ana container");
    }
}

pub struct Communicator {
    sender: zmq::Socket,
    receiver: zmq::Socket,
}

impl Communicator {
    pub fn new(ip_address: &str) -> io::Result<Communicator> {
        let context = zmq::Context::new();
        let sender = context.socket(zmq::PUSH)?;
        let receiver = context.socket(zmq::PULL)?;
        sender.connect(&format!("tcp://{}:{}", &ip_address, 8800))?;
        receiver.connect(&format!("tcp://{}:{}", &ip_address, 8801))?;
        Ok(Communicator { sender, receiver })
    }

    pub fn send(&self, data: &str) -> io::Result<()> {
        self.sender.send_str(data, 0)?;
        Ok(())
    }

    pub fn receive(&self) -> io::Result<String> {
        let data = self
            .receiver
            .recv_string(0)?
            .expect("What received is not a string");
        Ok(data)
    }
}

pub fn generate_judge_info<T: AsRef<path::Path>>(
    source_file: T,
    problem_file: T,
    spj_source_file: Option<T>,
) -> io::Result<mtp::JudgeInfo> {
    let source = mtp::Source {
        language: String::from("cpp.gxx"),
        code: String::from_utf8(fs::read(&source_file)?).unwrap(),
    };
    let mut problem: mtp::Problem = serde_json::from_reader(fs::File::open(&problem_file)?)?;
    if let Some(spj_source_file) = spj_source_file {
        problem.checker = mtp::Source {
            language: String::from("cpp.gxx"),
            code: String::from_utf8(fs::read(&spj_source_file)?).unwrap(),
        };
    }
    Ok(mtp::JudgeInfo {
        id: Uuid::new_v4().to_string(),
        source,
        problem,
    })
}

pub fn check_report_with_limit(
    report: &mtp::ReportInfo,
    id: &str,
    index: usize,
    status: &str,
    time: f64,
    memory: f64,
) {
    assert_eq!(report.id, id);
    assert_eq!(report.case_index, index);
    assert_eq!(report.status, status);
    assert!(report.time <= time * 1.01);
    assert!(report.memory <= memory * 1.01);
}
