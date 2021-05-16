use std::fs::File;
use std::io;
use std::io::Read;
use std::os::unix::fs as unix_fs;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use log;
use tokio::fs;
use tokio::sync::mpsc;

use crate::builder::Builder;
use crate::comparer::Comparer;
use crate::process::*;
use crate::runner::Runner;
use crate::workspace::{
    build::BuildDir,
    problem::{ProblemType, ResourceLimit},
    Workspace,
    runtime::RuntimeHolder
};

#[derive(Debug)]
pub struct Report {
    pub result: ResultType,
    pub usage: Option<Resource>,
    pub message: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ResultType {
    Accepted,
    WrongAnswer,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    RuntimeError,
    CompileError,
    SystemError,
}

#[derive(Clone, Copy, Debug)]
pub struct Resource {
    pub real_time: Duration,
    pub cpu_time: Duration,
    pub memory: usize,
}

impl From<ResourceLimit> for Resource {
    fn from(r: ResourceLimit) -> Resource {
        Resource {
            memory: r.memory,
            cpu_time: r.cpu_time,
            real_time: r.real_time,
        }
    }
}

pub async fn judge(
    workspace: Workspace,
    reporter: mpsc::UnboundedSender<Report>,
) -> io::Result<()> {
    log::debug!("Start judging workspace {}", workspace.as_path().display());
    log::debug!(
        "Start building source code in {}",
        workspace.build_dir().display()
    );
    let builder = match Builder::new(workspace.build_dir())? {
        Some(builder) => builder,
        None => {
            let res = Report {
                result: ResultType::SystemError,
                usage: None,
                message: String::from("The language of the source code is not supported"),
            };
            if reporter.send(res).is_err() {
                return Err(broken_channel());
            }
            return Ok(());
        }
    };
    let build_result = builder.build().await?;
    if !build_result.success {
        let res = Report {
            result: ResultType::CompileError,
            usage: None,
            message: String::from_utf8(build_result.stderr).unwrap_or_else(|_e| {
                String::from("Stderr of building process is not an valid utf8 string")
            }),
        };
        if reporter.send(res).is_err() {
            return Err(broken_channel());
        }
        return Ok(());
    }
    log::debug!(
        "Building source code in {} is finished",
        workspace.build_dir().display()
    );

    log::debug!(
        "Create runtime folder {}",
        workspace.build_dir().display()
    );


    // hold runtime folder
    let runtime_holder = RuntimeHolder::new(
        workspace.runtime_dir(), 
        workspace.config().runner.rootfs.as_ref()
    )?;

    log::debug!(
        "Start move compiled file to runtime directory {}",
        workspace.runtime_dir().display()
    );
    for file in workspace.build_dir().target_dir().read_dir()? {
        let src = file?.path();
        let dst = workspace.runtime_dir().join(
            src.strip_prefix(workspace.build_dir().target_dir())
                .unwrap(),
        );
        // skip directory
        if src.is_dir() {
            continue;
        }
        fs::copy(src, dst).await?;
    }
    log::debug!(
        "All compiled file has been moved to runtime directory {}",
        workspace.runtime_dir().display()
    );

    log::debug!("Start run program in {}", workspace.runtime_dir().display());
    let problem_dir = workspace.problem_dir();
    match problem_dir.config().problem_type {
        ProblemType::Normal => {
            for case in workspace.problem_dir().cases() {
                let runtime_dir = workspace.runtime_dir();
                if runtime_dir.input_file().exists() {
                    fs::remove_file(runtime_dir.input_file()).await?;
                }
                if runtime_dir.output_file().exists() {
                    fs::remove_file(runtime_dir.output_file()).await?;
                }
                let runner_config = &workspace.config().runner;
                log::debug!("Symlink the input file {}", case.input_file().display());
                unix_fs::symlink(case.input_file(), runtime_dir.input_file())?;
                log::debug!("Run the program in {}", runtime_dir.display());
                let mut child = Runner::new(&runtime_dir, runner_config)?
                    .stdin(File::open(runtime_dir.input_file())?)
                    .stdout(File::create(runtime_dir.output_file())?)
                    .stderr(Stdio::piped())
                    .spawn()?;
                log::debug!(
                    "Wait the process and get the result {}",
                    runtime_dir.display()
                );
                let start_time = Instant::now();
                let exit_status = child.timeout(problem_dir.config().limit.real_time)?;
                let resource_usage = {
                    let real_time = start_time.elapsed();
                    let (memory, cpu_time) = child.get_resource_usage()?;
                    Resource {
                        memory,
                        cpu_time,
                        real_time,
                    }
                };
                let stderr = child.stderr();
                log::debug!("Generate the process report of {}, {:?}", runtime_dir.display(), &resource_usage);

                let mut message = String::new();

                let result_type = if resource_usage.memory >= problem_dir.config().limit.memory {
                    ResultType::MemoryLimitExceeded
                } else if resource_usage.real_time > problem_dir.config().limit.real_time
                    || resource_usage.cpu_time > problem_dir.config().limit.cpu_time
                {
                    ResultType::TimeLimitExceeded
                } else if !exit_status.success() {
                    let mut buffer = vec![0; 1024];
                    if let Some(out) = stderr {
                        out.read(buffer.as_mut()); 
                        message = String::from_utf8_lossy(buffer.as_slice()).to_string();
                    }
                    ResultType::RuntimeError
                } else if Comparer::new(
                    problem_dir
                        .config()
                        .ignore_white_space_at_eol
                        .unwrap_or(true),
                    problem_dir
                        .config()
                        .ignore_empty_line_at_eof
                        .unwrap_or(true),
                )
                .compare_files(runtime_dir.output_file(), case.answer_file())
                .await?
                {
                    ResultType::Accepted
                } else {
                    ResultType::WrongAnswer
                };

                let res = Report {
                    result: result_type,
                    usage: Some(resource_usage),
                    message,
                };
                if reporter.send(res).is_err() {
                    return Err(broken_channel());
                }
            }
        }
        ProblemType::SpecialJudge => {
            let spj_dir = BuildDir::from_path(workspace.problem_dir().extern_program())?;
            let spj_builder = Builder::new(&spj_dir)?;
            let spj_builder = match spj_builder {
                Some(spj_builder) => spj_builder,
                None => {
                    let res = Report {
                        result: ResultType::SystemError,
                        usage: None,
                        message: String::from("The special judge of the problem is missing."),
                    };
                    if reporter.send(res).is_err() {
                        return Err(broken_channel());
                    }
                    return Ok(());
                }
            };
            let spj_build_result = spj_builder.build().await?;
            if !spj_build_result.success {
                let res = Report {
                    result: ResultType::SystemError,
                    usage: None,
                    message: String::from("Failed to build the special judge of the problem."),
                };
                if reporter.send(res).is_err() {
                    return Err(broken_channel());
                }
                return Ok(());
            }

            for case in workspace.problem_dir().cases() {
                let runtime_dir = workspace.runtime_dir();
                if runtime_dir.input_file().exists() {
                    fs::remove_file(runtime_dir.input_file()).await?;
                }
                if runtime_dir.output_file().exists() {
                    fs::remove_file(runtime_dir.output_file()).await?;
                }
                let runner_config = &workspace.config().runner;
                log::debug!("Symlink the input file {}", case.input_file().display());
                unix_fs::symlink(case.input_file(), runtime_dir.input_file())?;
                log::debug!("Run the program in {}", runtime_dir.display());
                let mut child = Runner::new(&runtime_dir, runner_config)?
                    .stdin(File::open(runtime_dir.input_file())?)
                    .stdout(File::create(runtime_dir.output_file())?)
                    .stderr(Stdio::piped())
                    .spawn()?;
                log::debug!(
                    "Wait the process and get the result {}",
                    runtime_dir.display()
                );
                let start_time = Instant::now();
                let exit_status = child.timeout(problem_dir.config().limit.real_time)?;
                let resource_usage = {
                    let real_time = start_time.elapsed();
                    let (memory, cpu_time) = child.get_resource_usage()?;
                    Resource {
                        memory,
                        cpu_time,
                        real_time,
                    }
                };
                log::debug!("Generate the process report of {}", runtime_dir.display());

                let result_type = if resource_usage.memory >= problem_dir.config().limit.memory {
                    ResultType::MemoryLimitExceeded
                } else if resource_usage.real_time > problem_dir.config().limit.real_time
                    || resource_usage.cpu_time > problem_dir.config().limit.cpu_time
                {
                    ResultType::TimeLimitExceeded
                } else if !exit_status.success() {
                    ResultType::RuntimeError
                } else if Command::new(spj_dir.executable_file())
                    .arg(case.input_file())
                    .arg(runtime_dir.output_file())
                    .arg(case.answer_file())
                    .spawn()?
                    .wait()?
                    .success()
                {
                    ResultType::Accepted
                } else {
                    ResultType::WrongAnswer
                };

                let res = Report {
                    result: result_type,
                    usage: Some(resource_usage),
                    message: String::new(),
                };
                if reporter.send(res).is_err() {
                    return Err(broken_channel());
                }
            }
        }
        ProblemType::Interactive => unimplemented!("TODO: Interactive support"),
    }

    // depress unsed variable warning
    drop(runtime_holder);
    Ok(())
}

fn broken_channel() -> io::Error {
    io::Error::new(
        io::ErrorKind::BrokenPipe,
        "Failed to send any more report through judge channel",
    )
}
