#[path = "../../src/rpc/"]
mod rpc {
    include!("../../src/rpc/mod.rs");
    pub use rpc_grpc::AnaClient;
}

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::Duration;

use ana::start_server;
use futures::{Future, Stream};
use grpcio::{ChannelBuilder, Environment, Server};
use liboj::structures::*;
use serde::{Deserialize, Serialize};
use serde_json;

use rpc::AnaClient;

pub const BYTES_PER_MB: usize = 1024 * 1024;

pub const SOURCE_AC: &str = include_str!("data/source.cpp");
pub const SOURCE_CE: &str = include_str!("data/source.ce.cpp");
pub const SOURCE_MLE: &str = include_str!("data/source.mle.cpp");
pub const SOURCE_RE: &str = include_str!("data/source.re.cpp");
pub const SOURCE_TLE: &str = include_str!("data/source.tle.cpp");
pub const SOURCE_WA: &str = include_str!("data/source.wa.cpp");

pub fn generate_judge_task(source: &str, problem: &str) -> Task {
    Task {
        source: Source {
            language: "cpp.g++".to_string(),
            code: source.to_owned(),
        },
        problem: {
            let problem: JsonProblem = serde_json::from_str(&problem).unwrap();
            match problem {
                JsonProblem::Normal {
                    time_limit,
                    memory_limit,
                    test_cases,
                } => Problem::Normal {
                    limit: Resource {
                        cpu_time: Duration::from_nanos(time_limit),
                        real_time: Duration::from_nanos(time_limit * 2),
                        memory: memory_limit as usize,
                    },
                    cases: test_cases
                        .into_iter()
                        .map(|case| TestCase {
                            input: case.input,
                            answer: case.answer,
                        })
                        .collect(),
                },
                JsonProblem::Special {
                    time_limit,
                    memory_limit,
                    test_cases,
                    spj,
                } => Problem::Special {
                    limit: Resource {
                        cpu_time: Duration::from_nanos(time_limit),
                        real_time: Duration::from_nanos(time_limit * 2),
                        memory: memory_limit as usize,
                    },
                    cases: test_cases
                        .into_iter()
                        .map(|case| TestCase {
                            input: case.input,
                            answer: case.answer,
                        })
                        .collect(),
                    spj,
                },
            }
        },
    }
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "Source")]
struct JsonSource {
    language: String,
    code: String,
}

#[derive(Serialize, Deserialize)]
struct JsonTestCase {
    input: String,
    answer: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum JsonProblem {
    Normal {
        time_limit: u64,   // NS
        memory_limit: u64, // Bytes
        test_cases: Vec<JsonTestCase>,
    },
    Special {
        time_limit: u64,   // NS
        memory_limit: u64, // Bytes
        test_cases: Vec<JsonTestCase>,
        #[serde(with = "JsonSource")]
        spj: Source,
    },
}

pub struct Judge {
    client: AnaClient,
    server: Server,
}

impl Judge {
    pub fn new() -> Judge {
        let server = start_server(1, IpAddr::V4(Ipv4Addr::LOCALHOST), 8000);
        let environment = Arc::new(Environment::new(1));
        let channel = ChannelBuilder::new(environment).connect("localhost:8000");
        let client = AnaClient::new(channel);
        Judge { client, server }
    }

    pub fn judge(&self, task: Task) -> Vec<Report> {
        self.client
            .judge(&task.into())
            .expect("Failed to call rpc")
            .map(|report| -> Report { report.into() })
            .collect()
            .wait()
            .unwrap()
    }
}

impl Drop for Judge {
    fn drop(&mut self) {
        self.server
            .shutdown()
            .wait()
            .expect("Failed to shutdown server");
    }
}
