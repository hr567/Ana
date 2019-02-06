use std::fs;
use std::path;
use std::thread;
use std::time;

use libc;
use tokio::prelude::*;
use tokio_threadpool;
use unshare;

mod cgroup;

#[derive(Debug)]
pub struct LaunchResult {
    pub exit_code: i32,
    pub real_time_usage: time::Duration,
    pub cpu_time_usage: u64,
    pub memory_usage: u64,
    pub tle_flag: bool,
    pub mle_flag: bool,
}

pub fn launch(
    name: &str,
    executable_file: &path::Path,
    input_file: &path::Path,
    output_file: &path::Path,
    time_limit: u64,
    memory_limit: u64,
) -> impl Future<Item = LaunchResult, Error = ()> {
    let cg = cgroup::Cgroup::new(&name, time_limit, memory_limit);
    let mut child = unshare::Command::new(&executable_file);
    let cgroup_hook = {
        let cpu_procs = cg.cpu_cgroup_path().join("cgroup.procs");
        let memory_procs = cg.memory_cgroup_path().join("cgroup.procs");
        move |pid: u32| {
            fs::write(&cpu_procs, format!("{}", pid)).unwrap();
            fs::write(&memory_procs, format!("{}", pid)).unwrap();
        }
    };
    child.before_unfreeze(move |pid| {
        cgroup_hook(pid);
        Ok(())
    });
    child
        .env_clear()
        .unshare(&[
            unshare::Namespace::Cgroup,
            unshare::Namespace::Net,
            unshare::Namespace::Pid,
            unshare::Namespace::User,
            unshare::Namespace::Ipc,
        ])
        // .uid(65534)
        // .gid(65534)
        .arg0("main")
        .stdin(unshare::Stdio::from_file(
            fs::File::open(&input_file).expect("Failed to open input file"),
        ))
        .stdout(unshare::Stdio::from_file(
            fs::File::create(&output_file).expect("Failed to create output file"),
        ));
    LaunchingTask {
        cg,
        child: child.spawn().expect("Failed to execute program"),
        start_time: time::Instant::now(),
        deadline: time::Instant::now() + time::Duration::from_nanos(time_limit * 5),
    }
}

pub struct LaunchingTask {
    cg: cgroup::Cgroup,
    child: unshare::Child,
    start_time: time::Instant,
    deadline: time::Instant,
}

impl Future for LaunchingTask {
    type Item = LaunchResult;
    type Error = ();
    fn poll(&mut self) -> Poll<LaunchResult, ()> {
        let child_pid = self.child.pid();
        let deadline = self.deadline;
        thread::spawn(move || {
            thread::sleep(deadline.duration_since(time::Instant::now()));
            unsafe {
                libc::kill(child_pid, libc::SIGKILL);
            }
        });
        match tokio_threadpool::blocking(|| self.child.wait().unwrap()) {
            Ok(Async::Ready(status)) => Ok(Async::Ready(LaunchResult {
                exit_code: status.code().unwrap_or(-1),
                real_time_usage: time::Instant::now() - self.start_time,
                cpu_time_usage: self.cg.get_cpu_time_usage(),
                memory_usage: self.cg.get_memory_usage(),
                tle_flag: self.cg.is_time_limit_exceeded(),
                mle_flag: self.cg.is_memory_limit_exceeded(),
            })),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(_) => Err(()),
        }
    }
}
