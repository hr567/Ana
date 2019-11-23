use std::fs;
use std::fs::File;
use std::io;
use std::process::Stdio;
use std::time::{Duration, Instant};

use futures::prelude::*;

use crate::builder::Builder;
use crate::comparer::Comparer;
use crate::process::*;
use crate::runner::Runner;
use crate::workspace::{
    config::{ProblemType, ResourceLimit},
    Workspace,
};

pub struct Report {
    pub result: ResultType,
    pub usage: Option<Resource>,
    pub message: String,
}

pub enum ResultType {
    Accepted,
    WrongAnswer,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    RuntimeError,
    CompileError,
    SystemError,
}

#[derive(Clone, Copy)]
pub struct Resource {
    pub real_time: Duration,
    pub cpu_time: Duration,
    pub memory: usize,
}

impl From<ResourceLimit> for Resource {
    fn from(r: ResourceLimit) -> Resource {
        Resource {
            memory: r.memory,
            cpu_time: Duration::from_nanos(r.cpu_time),
            real_time: Duration::from_nanos(r.real_time),
        }
    }
}

pub async fn judge(workspace: Workspace) -> io::Result<Box<dyn Stream<Item = Report>>> {
    let config = workspace.read_config()?;

    let builder = match Builder::new(workspace.build_dir(), config.builder)? {
        Some(builder) => builder,
        None => {
            let res: Box<dyn Stream<Item = Report>> = Box::new(stream::once(async {
                Report {
                    result: ResultType::SystemError,
                    usage: None,
                    message: String::from("The language of the source code is not supported"),
                }
            }));
            return Ok(res);
        }
    };
    let build_result = builder.build().await?;
    if !build_result.success {
        let res: Box<dyn Stream<Item = Report>> = Box::new(stream::once(async {
            Report {
                result: ResultType::CompileError,
                usage: None,
                message: String::from_utf8(build_result.stderr).unwrap_or(String::from(
                    "Stderr of building process is not an valid utf8 string",
                )),
            }
        }));
        return Ok(res);
    }

    for file in workspace.build_dir().target_dir().read_dir()? {
        fs::copy(file?.path(), workspace.runtime_dir())?;
    }

    let resource_limit = Resource::from(config.problem.limit);

    match config.problem.r#type {
        ProblemType::Normal => {
            let it = workspace
                .problem_dir()
                .cases()
                .map(async move |case| -> io::Result<Report> {
                    let config = workspace.read_config()?;
                    let runtime_dir = workspace.runtime_dir();
                    runtime_dir.activate_case(&case)?;
                    let mut child = Runner::new(&runtime_dir, config.runner.unwrap_or_default())?
                        .stdin(File::open(runtime_dir.input_file())?)
                        .stdout(File::create(runtime_dir.output_file())?)
                        .stderr(Stdio::piped())
                        .spawn()?;
                    let start_time = Instant::now();
                    let exit_status = child.timeout(resource_limit.real_time)?;
                    let real_time = start_time.elapsed();
                    let (memory, cpu_time) = child.get_resource_usage()?;
                    let resource_usage = Resource {
                        memory,
                        cpu_time,
                        real_time: real_time,
                    };

                    let result_type = if resource_usage.memory >= resource_limit.memory {
                        ResultType::MemoryLimitExceeded
                    } else if resource_usage.real_time > resource_limit.real_time
                        || resource_usage.cpu_time > resource_limit.cpu_time
                    {
                        ResultType::TimeLimitExceeded
                    } else if !exit_status.success() {
                        ResultType::RuntimeError
                    } else if Comparer::new(
                        config.problem.ignore_white_space_at_eol.unwrap_or(true),
                        config.problem.ignore_empty_line_at_eof.unwrap_or(true),
                    )
                    .compare_files(runtime_dir.output_file(), case.answer_file())
                    .await
                    .unwrap_or(false)
                    {
                        ResultType::Accepted
                    } else {
                        ResultType::WrongAnswer
                    };

                    Ok(Report {
                        result: result_type,
                        usage: Some(resource_usage),
                        message: String::new(),
                    })
                })
                .map(async move |res| match res.await {
                    Ok(report) => report,
                    Err(e) => Report {
                        result: ResultType::SystemError,
                        usage: None,
                        message: format!("IO Error: {}", e),
                    },
                });
            let res: Box<dyn Stream<Item = Report>> = Box::new(stream::iter(it));
            Ok(res)
        }
        ProblemType::SpecialJudge => unimplemented!("TODO: Special judge support"),
        ProblemType::Interactive => unimplemented!("TODO: Interactive support"),
    }
}
