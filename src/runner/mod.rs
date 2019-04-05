use std::fs;
use std::path;
use std::thread;
use std::time;

use nix;
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
    chroot_dir: &path::Path,
    input_file: &path::Path,
    output_file: &path::Path,
    time_limit: u64,
    memory_limit: u64,
) -> impl Future<Item = LaunchResult, Error = ()> {
    let cg = cgroup::Cgroup::new(time_limit, memory_limit);
    let mut child = unshare::Command::new("/main");
    let child_hook = {
        let cpu_procs = cg.cpu_cgroup_path().join("cgroup.procs");
        let memory_procs = cg.memory_cgroup_path().join("cgroup.procs");
        move |pid: u32| {
            fs::write(&cpu_procs, pid.to_string())
                .expect("Failed to write to time cgroup processes");
            fs::write(&memory_procs, pid.to_string())
                .expect("Failed to write to memory cgroup processes");
            thread::spawn(move || {
                thread::sleep(time::Duration::from_nanos(time_limit / 2 * 3));
                let _res = nix::sys::signal::kill(
                    nix::unistd::Pid::from_raw(pid as i32),
                    Some(nix::sys::signal::SIGKILL),
                );
            });
            Ok(())
        }
    };
    child.before_unfreeze(child_hook);
    child
        .env_clear()
        .arg0("/main")
        // .uid(65534)
        // .gid(65534)
        .unshare(&[
            unshare::Namespace::Cgroup,
            unshare::Namespace::Ipc,
            unshare::Namespace::Mount,
            unshare::Namespace::Net,
            unshare::Namespace::Pid,
            unshare::Namespace::User,
            unshare::Namespace::Uts,
        ])
        .current_dir("/")
        .chroot_dir(chroot_dir)
        .stdin(unshare::Stdio::from_file(
            fs::File::open(input_file).unwrap(),
        ))
        .stdout(unshare::Stdio::from_file(
            fs::File::create(output_file).unwrap(),
        ));
    LaunchFuture {
        cg,
        child,
        start_time: time::Instant::now(),
    }
}

pub struct LaunchFuture {
    cg: cgroup::Cgroup,
    child: unshare::Command,
    start_time: time::Instant,
}

unsafe impl Send for LaunchFuture {}

impl Future for LaunchFuture {
    type Item = LaunchResult;
    type Error = ();
    fn poll(&mut self) -> Poll<LaunchResult, ()> {
        match tokio_threadpool::blocking(|| match self.child.status() {
            Ok(res) => res,
            Err(e) => panic!("Failed to spawn child process: {}", e),
        }) {
            Ok(Async::Ready(status)) => {
                let res = LaunchResult {
                    exit_code: status.code().unwrap_or(-1),
                    real_time_usage: time::Instant::now() - self.start_time,
                    cpu_time_usage: self.cg.get_cpu_time_usage(),
                    memory_usage: self.cg.get_memory_usage(),
                    tle_flag: self.cg.is_time_limit_exceeded(),
                    mle_flag: self.cg.is_memory_limit_exceeded(),
                };
                Ok(Async::Ready(res))
            }
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(_) => Err(()),
        }
    }
}
