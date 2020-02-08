use std::time::Duration;

use crate::judge;
use crate::workspace;

tonic::include_proto!("rpc");

impl From<judge::Report> for Report {
    fn from(report: judge::Report) -> Report {
        Report {
            result: self::report::ResultType::from(report.result) as i32,
            usage: report.usage.map(Resource::from),
            message: Some(report.message),
        }
    }
}

impl From<judge::ResultType> for report::ResultType {
    fn from(res: judge::ResultType) -> report::ResultType {
        use judge::ResultType::*;
        match res {
            Accepted => report::ResultType::Accepted,
            WrongAnswer => report::ResultType::WrongAnswer,
            TimeLimitExceeded => report::ResultType::TimeLimitExceeded,
            MemoryLimitExceeded => report::ResultType::MemoryLimitExceeded,
            RuntimeError => report::ResultType::RuntimeError,
            CompileError => report::ResultType::CompileError,
            SystemError => report::ResultType::SystemError,
        }
    }
}

impl From<judge::Resource> for Resource {
    fn from(resource: judge::Resource) -> Resource {
        use prost_types::Duration;

        Resource {
            real_time: Some(Duration::from(resource.real_time)),
            cpu_time: Some(Duration::from(resource.cpu_time)),
            memory: Some(resource.memory as u64),
        }
    }
}

impl From<Resource> for workspace::problem::ResourceLimit {
    fn from(resource: Resource) -> workspace::problem::ResourceLimit {
        let real_time = match resource.real_time {
            Some(real_time) => Duration::new(real_time.seconds as u64, real_time.nanos as u32),
            None => Duration::from_secs(0),
        };
        let cpu_time = match resource.cpu_time {
            Some(cpu_time) => Duration::new(cpu_time.seconds as u64, cpu_time.nanos as u32),
            None => Duration::from_secs(0),
        };
        workspace::problem::ResourceLimit {
            real_time,
            cpu_time,
            memory: resource.memory.unwrap_or(0) as usize,
        }
    }
}

impl From<RunnerConfig> for workspace::RunnerConfig {
    fn from(_config: RunnerConfig) -> workspace::RunnerConfig {
        unimplemented!("TODO: RunnerConfig is unavailable now")
    }
}

impl From<&RunnerConfig> for workspace::RunnerConfig {
    fn from(_config: &RunnerConfig) -> workspace::RunnerConfig {
        unimplemented!("TODO: RunnerConfig is unavailable now")
    }
}
