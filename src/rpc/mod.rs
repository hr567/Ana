mod rpc;
mod rpc_grpc;

use std::time::Duration;

use liboj::structures::*;

pub use rpc::{Report as RpcReport, Task as RpcTask};
pub use rpc_grpc::{create_ana, Ana};

impl From<Task> for RpcTask {
    fn from(task: Task) -> RpcTask {
        let mut res = RpcTask::new();
        res.set_source(task.source.into());
        res.set_problem(task.problem.into());
        res
    }
}

impl Into<Task> for RpcTask {
    fn into(self) -> Task {
        Task {
            source: self.source.unwrap().into(),
            problem: self.problem.unwrap().into(),
        }
    }
}

impl From<Problem> for rpc::Problem {
    fn from(problem: Problem) -> rpc::Problem {
        let mut res = rpc::Problem::new();
        match problem {
            Problem::Normal { limit, cases } => {
                let mut inner_problem = rpc::NormalProblem::new();
                inner_problem.set_limit(limit.into());
                inner_problem.set_cases(cases.into_iter().map(|case| case.into()).collect());
                res.set_normal(inner_problem);
            }
            Problem::Special { limit, cases, spj } => {
                let mut inner_problem = rpc::SpecialProblem::new();
                inner_problem.set_limit(limit.into());
                inner_problem.set_cases(cases.into_iter().map(|case| case.into()).collect());
                inner_problem.set_spj(spj.into());
                res.set_special(inner_problem);
            }
        }
        res
    }
}

impl Into<Problem> for rpc::Problem {
    fn into(mut self) -> Problem {
        if self.has_normal() {
            let rpc::NormalProblem { limit, cases, .. } = self.take_normal();
            Problem::Normal {
                limit: limit.unwrap().into(),
                cases: cases.into_iter().map(|case| case.into()).collect(),
            }
        } else if self.has_special() {
            let rpc::SpecialProblem {
                limit, cases, spj, ..
            } = self.take_special();
            Problem::Special {
                limit: limit.unwrap().into(),
                cases: cases.into_iter().map(|case| case.into()).collect(),
                spj: spj.unwrap().into(),
            }
        } else {
            unreachable!("Only two types of problem now")
        }
    }
}

impl From<TestCase> for rpc::TestCase {
    fn from(case: TestCase) -> rpc::TestCase {
        let mut res = rpc::TestCase::new();
        res.set_input(case.input);
        res.set_answer(case.answer);
        res
    }
}

impl Into<TestCase> for rpc::TestCase {
    fn into(self) -> TestCase {
        TestCase {
            input: self.input,
            answer: self.answer,
        }
    }
}

impl From<Source> for rpc::Source {
    fn from(source: Source) -> rpc::Source {
        let mut res = rpc::Source::new();
        res.set_language(source.language);
        res.set_code(source.code);
        res
    }
}

impl Into<Source> for rpc::Source {
    fn into(self) -> Source {
        Source {
            language: self.language,
            code: self.code,
        }
    }
}

impl From<Resource> for rpc::Resource {
    fn from(resource: Resource) -> rpc::Resource {
        let mut res = rpc::Resource::new();
        res.set_real_time(resource.cpu_time.as_nanos() as u64);
        res.set_cpu_time(resource.cpu_time.as_nanos() as u64);
        res.set_memory(resource.memory as u64);
        res
    }
}

impl Into<Resource> for rpc::Resource {
    fn into(self) -> Resource {
        Resource {
            real_time: Duration::from_nanos(self.real_time),
            cpu_time: Duration::from_nanos(self.cpu_time),
            memory: self.memory as usize,
        }
    }
}

impl From<Report> for RpcReport {
    fn from(report: Report) -> RpcReport {
        use Report::*;

        let mut res = RpcReport::new();
        match report {
            Accepted { resource_usage } => {
                res.set_result(rpc::Report_Result::Accepted);
                res.set_usage(resource_usage.into());
            }
            WrongAnswer => {
                res.set_result(rpc::Report_Result::WrongAnswer);
            }
            TimeLimitExceeded => {
                res.set_result(rpc::Report_Result::TimeLimitExceeded);
            }
            MemoryLimitExceeded => {
                res.set_result(rpc::Report_Result::MemoryLimitExceeded);
            }
            RuntimeError => {
                res.set_result(rpc::Report_Result::RuntimeError);
            }
            CompileError => {
                res.set_result(rpc::Report_Result::CompileError);
            }
            SystemError => {
                res.set_result(rpc::Report_Result::SystemError);
            }
        }
        res
    }
}

impl Into<Report> for RpcReport {
    fn into(self) -> Report {
        use rpc::Report_Result::*;

        match self.get_result() {
            Accepted => Report::Accepted {
                resource_usage: self.usage.unwrap().into(),
            },
            WrongAnswer => Report::WrongAnswer,
            TimeLimitExceeded => Report::TimeLimitExceeded,
            MemoryLimitExceeded => Report::MemoryLimitExceeded,
            RuntimeError => Report::RuntimeError,
            CompileError => Report::CompileError,
            SystemError => Report::SystemError,
        }
    }
}
