mod rpc;
mod workspace;

use std::iter::{once, Iterator};
use std::net::IpAddr;
use std::path::Path;
use std::sync::Arc;

use futures::{stream::iter_ok, Future, Sink, Stream};
use grpcio::{Environment, RpcContext, Server, ServerBuilder, ServerStreamingSink, WriteFlags};
use liboj::{checker, compiler::Compiler, runner, structures::*};
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
        ctx.spawn(
            sink.send_all(
                iter_ok(judge(task.into()))
                    .map(|report| (RpcReport::from(report), WriteFlags::default()))
                    .map_err(|()| grpcio::Error::RpcFinished(None)), // Unreachable here. No error in the iterator.
            )
            .map(drop)
            .map_err(|e| panic!("gRPC error: {}", e)),
        );
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
    let compile_success = Compiler::new(&language)
        .expect("The language is not support")
        .compile(&source, &work_dir.runtime_dir().executable_file())
        .expect("Failed to run compiler");

    if !compile_success {
        return Box::new(once(Report::CompileError));
    }

    match problem {
        Problem::Normal { limit, .. } => {
            Box::new(work_dir.problem_dir().test_case_dirs().into_iter().map(
                move |test_case_dir| {
                    let runner_report = runner::Runner::new(
                        "/main",
                        test_case_dir.input_file(),
                        test_case_dir.output_file(),
                    )
                    .chroot(work_dir.runtime_dir())
                    .resource_limit(limit)
                    .run()
                    .expect("Failed to run program");
                    generate_report(
                        limit,
                        runner_report,
                        &test_case_dir.input_file(),
                        &test_case_dir.output_file(),
                        &test_case_dir.answer_file(),
                        Option::<&Path>::None,
                    )
                },
            ))
        }
        Problem::Special { limit, spj, .. } => {
            let spj_compile_success = {
                let Source { language, .. } = &spj;
                Compiler::new(&language)
                    .expect("The language is not support")
                    .compile(&spj, &work_dir.problem_dir().spj_file())
                    .expect("Failed to run compiler")
            };
            assert!(spj_compile_success, "Failed to compile spj");

            Box::new(work_dir.problem_dir().test_case_dirs().into_iter().map(
                move |test_case_dir| {
                    let runner_report = runner::Runner::new(
                        "/main",
                        test_case_dir.input_file(),
                        test_case_dir.output_file(),
                    )
                    .chroot(work_dir.runtime_dir())
                    .resource_limit(limit)
                    .run()
                    .expect("Failed to run program");
                    generate_report(
                        limit,
                        runner_report,
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
    resource_limit: Resource,
    runner_report: runner::RunnerReport,
    input_file: impl AsRef<Path>,
    output_file: impl AsRef<Path>,
    answer_file: impl AsRef<Path>,
    spj: Option<impl AsRef<Path>>,
) -> Report {
    let runner::RunnerReport {
        exit_success,
        resource_usage,
    } = runner_report;

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
