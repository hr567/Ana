mod rpc;

use std::io;
use std::marker::Unpin;
use std::net::{IpAddr, SocketAddr};
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process;
use std::time::Duration;

use async_trait::async_trait;
use futures::executor;
use futures::prelude::*;
use rpc::ana_server as server;
use rpc::{Problem, Report, Task};
use tempfile;
use tokio::fs;
use tokio::prelude::*;
use tokio::runtime::{self, Runtime};
use tokio::sync::mpsc;
use toml;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use crate::judge;
use crate::workspace::{self, problem::Case, Workspace};

pub struct RpcServer {
    runtime: Runtime,
}

impl RpcServer {
    pub fn new(max_threads: usize) -> RpcServer {
        let runtime = runtime::Builder::new()
            .core_threads(max_threads)
            .enable_all()
            .build()
            .expect("Failed to create a runtime");
        RpcServer { runtime }
    }

    pub fn start(self, address: IpAddr, port: u16) -> ! {
        let srv = server::AnaServer::new(self);
        match executor::block_on(
            Server::builder()
                .add_service(srv)
                .serve(SocketAddr::new(address, port)),
        ) {
            Ok(()) => process::exit(0),
            Err(e) => panic!("Serve stopped. {}", e),
        }
    }
}

#[async_trait]
impl server::Ana for RpcServer {
    type JudgeStream =
        Pin<Box<dyn Stream<Item = Result<Report, Status>> + Send + Unpin + Sync + 'static>>;

    async fn judge(
        &self,
        request: Request<Task>,
    ) -> Result<Response<<Self as server::Ana>::JudgeStream>, Status> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.runtime.spawn(async move {
            let task = request.get_ref();
            let workspace = match task.construct_workspace().await {
                Ok(workspace) => workspace,
                Err(e) => {
                    let _ = tx.send(judge::Report {
                        result: judge::ResultType::SystemError,
                        usage: None,
                        message: format!("Failed to construct workspace. {}", e),
                    });
                    return;
                }
            };
            match judge::judge(workspace, tx.clone()).await {
                Ok(()) => {}
                Err(e) => {
                    let _ = tx.send(judge::Report {
                        result: judge::ResultType::SystemError,
                        usage: None,
                        message: format!("Failed to judge task. {}", e),
                    });
                }
            }
        });

        let res: Box<dyn Stream<Item = Result<Report, Status>> + Send + Sync + Unpin + 'static> =
            Box::new(rx.map(|report| Ok(Report::from(report))));
        Ok(Response::new(Pin::new(res)))
    }

    async fn cache(&self, _request: Request<Problem>) -> Result<Response<()>, Status> {
        todo!()
    }
}

#[async_trait]
trait RpcTask {
    async fn construct_workspace(&self) -> io::Result<Workspace>;
}

#[async_trait]
impl RpcTask for Task {
    async fn construct_workspace(&self) -> io::Result<Workspace> {
        let workspace = tempfile::tempdir()?;

        match &self.source {
            Some(source) => {
                construct_build_dir(
                    workspace.path().join("build"),
                    &source,
                    &self.language,
                    &self.build_script,
                    &self.build_timeout,
                )
                .await?;
            }
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Source field is missing",
                ));
            }
        }

        match &self.problem {
            Some(problem) => {
                construct_problem_dir(workspace.path().join("problem"), &problem).await?;
            }
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Problem field is missing",
                ));
            }
        }

        if let Some(runner) = &self.runner_config {
            // TODO: Runner config support
            let runner_config = workspace::RunnerConfig {
                command: runner.command.clone().map(PathBuf::from),
                args: Some(runner.args.clone()),
                cgroups: None,
                seccomp: None,
                namespaces: None,
            };
            fs::write(
                workspace.path().join("config.toml"),
                &toml::to_string(&runner_config).unwrap(),
            )
            .await?;
        }

        Ok(Workspace::from_path(workspace)?)
    }
}

async fn construct_build_dir<P: AsRef<Path>>(
    build_dir: P,
    source: &rpc::File,
    language: &Option<String>,
    build_script: &Option<rpc::File>,
    timeout: &Option<prost_types::Duration>,
) -> io::Result<()> {
    let build_dir = build_dir.as_ref();
    let mut builder_config = workspace::build::Config {
        source: PathBuf::from(&source.filename),
        language: None,
        build_script: None,
        timeout: None,
    };
    fs::File::create(build_dir.join(&source.filename))
        .await?
        .write_all(&source.content)
        .await?;
    if let Some(build_script) = &build_script {
        fs::File::create(build_dir.join(&build_script.filename))
            .await?
            .write_all(&build_script.content)
            .await?;
        builder_config.build_script = Some(PathBuf::from(&build_script.filename));
    } else if let Some(language) = &language {
        builder_config.language = Some(language.clone());
    }
    if let Some(timeout) = &timeout {
        builder_config.timeout = Some(Duration::new(timeout.seconds as u64, timeout.nanos as u32));
    }
    let builder_config = match toml::to_string(&builder_config) {
        Ok(config) => config,
        Err(e) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize builder's configuration. {}", e),
            ));
        }
    };
    fs::File::create(&build_dir.join("config.toml"))
        .await?
        .write_all(builder_config.as_bytes())
        .await?;

    Ok(())
}

async fn construct_problem_dir<P: AsRef<Path>>(
    problem_dir: P,
    problem: &rpc::Problem,
) -> io::Result<()> {
    let problem_dir = problem_dir.as_ref();

    let (resource_limit, problem) = if let Problem {
        id: _id,
        limit: Some(limit),
        problem: Some(problem),
    } = problem
    {
        (limit, problem)
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Some fields in problem are missing.",
        ));
    };

    match problem {
        rpc::problem::Problem::NormalProblem(rpc::problem::Normal {
            cases,
            ignore_white_space_at_eol,
            ignore_empty_line_at_eof,
            check_script,
        }) => {
            let problem_config = workspace::problem::Config {
                problem_type: workspace::problem::ProblemType::Normal,
                limit: resource_limit.clone().into(),
                ignore_white_space_at_eol: *ignore_white_space_at_eol,
                ignore_empty_line_at_eof: *ignore_empty_line_at_eof,
                extern_program: None,
            };
            fs::write(
                problem_dir.join("config.toml"),
                toml::to_string(&problem_config).unwrap(),
            )
            .await?;
            for (index, case) in cases.into_iter().enumerate() {
                let case_dir = Case::new(problem_dir.join(index.to_string()));
                fs::create_dir(case_dir.as_path()).await?;
                fs::write(case_dir.input_file(), &case.input).await?;
                fs::write(case_dir.answer_file(), &case.answer).await?;
            }
            if let Some(check_script) = check_script {
                fs::write(problem_dir.join("check.sh"), &check_script.content).await?;
            }
        }
        rpc::problem::Problem::SpjProblem(rpc::problem::SpecialJudge {
            cases,
            source,
            language,
            build_script,
            check_script,
        }) => {
            let spj_dir = problem_dir.join("extern_program");
            fs::create_dir(&spj_dir).await?;
            let source = match source {
                Some(source) => source,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "The source code of special judge is missing",
                    ));
                }
            };
            fs::write(spj_dir.join(&source.filename), &source.content).await?;

            let build_script = match build_script {
                Some(build_script) => {
                    let rpc::File {
                        filename: _,
                        content,
                    } = build_script;
                    fs::write(spj_dir.join("build.sh"), content).await?;
                    Some(spj_dir.join("build.sh"))
                }
                None => None,
            };

            let problem_config = workspace::problem::Config {
                problem_type: workspace::problem::ProblemType::SpecialJudge,
                limit: resource_limit.clone().into(),
                ignore_white_space_at_eol: None,
                ignore_empty_line_at_eof: None,
                extern_program: Some(workspace::problem::ExternProgram {
                    source: problem_dir.join("spj").join(&source.filename),
                    language: language.as_ref().map(String::from),
                    build_script,
                }),
            };
            fs::write(
                problem_dir.join("config.toml"),
                toml::to_string(&problem_config).unwrap(),
            )
            .await?;
            for (index, case) in cases.into_iter().enumerate() {
                let case_dir = Case::new(problem_dir.join(format!("{}", index)));
                fs::create_dir(case_dir.as_path()).await?;
                fs::write(case_dir.input_file(), &case.input).await?;
                fs::write(case_dir.answer_file(), &case.answer).await?;
            }
            if let Some(check_script) = check_script {
                fs::write(problem_dir.join("check.sh"), &check_script.content).await?;
            }
        }
        rpc::problem::Problem::InteractiveProblem { .. } => {
            unimplemented!("Interactive problem is not supported now")
        }
        rpc::problem::Problem::CachedProblem(cached_problem) => {
            unix_fs::symlink(cached_problem, problem_dir)?;
        }
    }

    Ok(())
}
