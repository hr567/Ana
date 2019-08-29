mod rpc;
mod workspace;

use std::fs::File;
use std::io;
use std::iter::{once, Iterator};
use std::net::IpAddr;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};

use executor::{cgroup::CommandExt as _, ChildExt as _, CommandExt as _};
use futures::{stream::iter_ok, Future, Sink, Stream};
use grpcio::{Environment, RpcContext, Server, ServerBuilder, ServerStreamingSink, WriteFlags};
use liboj::*;
use rpc::{create_ana, Ana, RpcReport, RpcTask};
use workspace::*;

pub fn start_server(judge_threads: usize, address: IpAddr, port: u16) -> Server {
    let env = Arc::new(Environment::new(judge_threads));
    let service = create_ana(AnaService);
    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .bind(address.to_string(), port)
        .build()
        .expect("Failed to start gRPC server");
    server.start();
    server
}

#[derive(Clone, Copy)]
pub struct AnaService;

impl Ana for AnaService {
    fn judge(&mut self, ctx: RpcContext<'_>, task: RpcTask, sink: ServerStreamingSink<RpcReport>) {
        let reports = judge(task.into());
        let reply = iter_ok(reports)
            .map(|report| (RpcReport::from(report), WriteFlags::default()))
            .map_err(|()| grpcio::Error::RpcFinished(None)); // Unreachable here. No error in the iterator.
        let sending = sink
            .send_all(reply)
            .map(drop)
            .map_err(|e| panic!("gRPC error: {}", e));
        ctx.spawn(sending);
    }
}

/// Judge the task and generate a list of reports
fn judge(task: Task) -> Box<dyn Iterator<Item = Report> + Send> {
    let work_dir = Workspace::new();
    work_dir
        .prepare_task(&task)
        .expect("Failed to prepare judge task");

    let Task { source, problem } = task;

    let Source { language, .. } = &source;
    let compiler_output = Compiler::new(&language)
        .expect("The language is not support")
        .compile(&source, &work_dir.runtime_dir().executable_file())
        .expect("Failed to run compiler");

    if !compiler_output.status.success() {
        return Box::new(once(Report::CompileError));
    }

    match problem {
        Problem::Normal { limit, .. } => {
            Box::new(work_dir.problem_dir().test_case_dirs().into_iter().map(
                move |test_case_dir| {
                    let input_file =
                        File::open(&test_case_dir.input_file()).expect("Failed to open input file");
                    let output_file = File::create(&test_case_dir.output_file())
                        .expect("Failed to create output file");
                    let cg_ctx = {
                        let mut res = executor::cgroup::Builder::default()
                            .build()
                            .expect("Failed to create new cgroup");
                        res.set_limit(limit).expect("Failed to set resource limit");
                        res
                    };
                    let start_time = Instant::now();

                    let exit_status = Command::new("/main")
                        .stdin(input_file)
                        .stdout(output_file)
                        .cgroup(cg_ctx.clone())
                        .chroot(work_dir.runtime_dir())
                        .spawn()
                        .expect("Failed to spawn child program")
                        .timeout(limit.real_time)
                        .expect("Failed to wait for child program");

                    let usage = Resource {
                        real_time: start_time.elapsed(),
                        ..cg_ctx
                            .get_usage()
                            .expect("Failed to get resource usage from cgroup")
                    };

                    generate_report(
                        exit_status.success(),
                        limit,
                        usage,
                        &test_case_dir.input_file(),
                        &test_case_dir.output_file(),
                        &test_case_dir.answer_file(),
                        Option::<&Path>::None,
                    )
                },
            ))
        }
        Problem::Special { limit, spj, .. } => {
            let spj_compiler_output = {
                let Source { language, .. } = &spj;
                Compiler::new(&language)
                    .expect("The language is not support")
                    .compile(&spj, &work_dir.problem_dir().spj_file())
                    .expect("Failed to run compiler")
            };
            assert!(
                spj_compiler_output.status.success(),
                "Failed to compile spj"
            );

            Box::new(work_dir.problem_dir().test_case_dirs().into_iter().map(
                move |test_case_dir| {
                    let input_file =
                        File::open(&test_case_dir.input_file()).expect("Failed to open input file");
                    let output_file = File::create(&test_case_dir.output_file())
                        .expect("Failed to create output file");
                    let cg_ctx = {
                        let mut res = executor::cgroup::Builder::default()
                            .build()
                            .expect("Failed to create new cgroup");
                        res.set_limit(limit).expect("Failed to set resource limit");
                        res
                    };
                    let start_time = Instant::now();

                    let exit_status = Command::new("/main")
                        .stdin(input_file)
                        .stdout(output_file)
                        .cgroup(cg_ctx.clone())
                        .chroot(work_dir.runtime_dir())
                        .spawn()
                        .expect("Failed to spawn child program")
                        .timeout(limit.real_time)
                        .expect("Failed to wait for child program");

                    let usage = Resource {
                        real_time: start_time.elapsed(),
                        ..cg_ctx
                            .get_usage()
                            .expect("Failed to get resource usage from cgroup")
                    };

                    generate_report(
                        exit_status.success(),
                        limit,
                        usage,
                        &test_case_dir.input_file(),
                        &test_case_dir.output_file(),
                        &test_case_dir.answer_file(),
                        Some(&work_dir.problem_dir().spj_file()),
                    )
                },
            ))
        }
    }
}

/// Generate the judge report using given data.
/// The order of different cases is important
/// because TLE or RE may be caused by MLE.
/// So check memory usage first.
/// If the cpu time usage being much smaller than
/// real time usage means that there are too many
/// threads working in one time or the program use sleep.
fn generate_report(
    exit_success: bool,
    resource_limit: Resource,
    resource_usage: Resource,
    input_file: impl AsRef<Path>,
    output_file: impl AsRef<Path>,
    answer_file: impl AsRef<Path>,
    spj: Option<impl AsRef<Path>>,
) -> Report {
    if resource_usage.memory >= resource_limit.memory {
        Report::MemoryLimitExceeded
    } else if resource_usage.real_time >= resource_limit.real_time
        || resource_usage.cpu_time >= resource_limit.cpu_time
    {
        Report::TimeLimitExceeded
    } else if !exit_success {
        Report::RuntimeError
    } else if match spj {
        Some(spj) => checker::Checker::default()
            .extern_program(spj.as_ref())
            .check_use_extern_program(
                input_file.as_ref(),
                output_file.as_ref(),
                answer_file.as_ref(),
            )
            .unwrap_or(false),
        None => checker::Checker::default()
            .compare_files(output_file.as_ref(), answer_file.as_ref())
            .unwrap_or(false),
    } {
        Report::Accepted { resource_usage }
    } else {
        Report::WrongAnswer
    }
}

trait ResourceController {
    fn set_limit(&mut self, limit: Resource) -> io::Result<()>;
    fn get_usage(&self) -> io::Result<Resource>;
}

impl ResourceController for executor::cgroup::Context {
    fn set_limit(&mut self, limit: Resource) -> io::Result<()> {
        let cpu_controller = self.cpu_controller().unwrap();
        let memory_controller = self.memory_controller().unwrap();

        let real_time = limit.real_time;
        let cpu_time = limit.cpu_time;

        let period = Duration::from_secs(1);
        let quota = {
            let real_time = real_time.as_micros() as u32;
            let cpu_time = cpu_time.as_micros() as u32;
            period * cpu_time / real_time
        };

        cpu_controller.period().write(&period)?;
        cpu_controller.quota().write(&quota)?;
        memory_controller.limit_in_bytes().write(&limit.memory)?;

        Ok(())
    }

    fn get_usage(&self) -> io::Result<Resource> {
        let res = Resource {
            cpu_time: self
                .cpuacct_controller()
                .expect("Failed to get cpuacct controller")
                .usage()?,
            real_time: Duration::from_secs(0),
            memory: self
                .memory_controller()
                .expect("Failed to get memory controller")
                .max_usage_in_bytes()?,
        };
        Ok(res)
    }
}
