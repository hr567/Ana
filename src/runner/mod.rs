use std::ffi;
use std::fs;
use std::io;
use std::os::unix::io::{AsRawFd as _, IntoRawFd as _};
use std::path;
use std::thread;
use std::time;

use nix;
use tokio::prelude::*;
use tokio_threadpool;
mod cgroup;

#[derive(Debug)]
pub struct LaunchResult {
    pub exit_success: bool,
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

    match nix::unistd::fork().expect("Failed to fork a child process") {
        nix::unistd::ForkResult::Parent { child: pid } => {
            fs::write(&cg.cpu_cgroup_path().join("cgroup.procs"), pid.to_string())
                .expect("Failed to write to time cgroup processes");
            fs::write(
                &cg.memory_cgroup_path().join("cgroup.procs"),
                pid.to_string(),
            )
            .expect("Failed to write to memory cgroup processes");
            thread::spawn(move || {
                thread::sleep(time::Duration::from_nanos(time_limit / 2 * 3));
                nix::sys::signal::kill(pid, nix::sys::signal::SIGKILL).unwrap_or_else(|e| {
                    assert_eq!(
                        e,
                        nix::Error::Sys(nix::errno::Errno::ESRCH),
                        "Failed to kill long running child process"
                    );
                });
            });
            LaunchFuture {
                cg,
                pid,
                start_time: time::Instant::now(),
            }
        }
        nix::unistd::ForkResult::Child => {
            nix::sched::unshare(
                nix::sched::CloneFlags::empty()
                    | nix::sched::CloneFlags::CLONE_FILES
                    | nix::sched::CloneFlags::CLONE_FS
                    | nix::sched::CloneFlags::CLONE_NEWCGROUP
                    | nix::sched::CloneFlags::CLONE_NEWIPC
                    | nix::sched::CloneFlags::CLONE_NEWNET
                    | nix::sched::CloneFlags::CLONE_NEWNS
                    | nix::sched::CloneFlags::CLONE_NEWPID
                    | nix::sched::CloneFlags::CLONE_NEWUSER
                    | nix::sched::CloneFlags::CLONE_NEWUTS
                    | nix::sched::CloneFlags::CLONE_SYSVSEM,
            )
            .expect("Failed to unshare namespace");
            let input_fd = fs::File::open(input_file)
                .expect("Failed to open input file")
                .into_raw_fd();
            let output_fd = fs::File::create(output_file)
                .expect("Failed to create output file")
                .into_raw_fd();
            nix::unistd::dup2(input_fd, io::stdin().as_raw_fd()).expect("Failed to dup stdin");
            nix::unistd::dup2(output_fd, io::stdout().as_raw_fd()).expect("Failed to dup stdout");
            nix::unistd::close(input_fd).expect("Failed to close input file");
            nix::unistd::close(output_fd).expect("Failed to close output file");

            nix::unistd::chroot(chroot_dir).expect("Failed to chroot");
            nix::unistd::chdir("/").expect("Failed to chdir");
            nix::unistd::execvpe(&ffi::CString::new("/main").unwrap(), &[], &[])
                .expect("Failed to exec child process");

            unreachable!("Not reachable after exec")
        }
    }
}

pub struct LaunchFuture {
    cg: cgroup::Cgroup,
    pid: nix::unistd::Pid,
    start_time: time::Instant,
}

unsafe impl Send for LaunchFuture {}

impl Future for LaunchFuture {
    type Item = LaunchResult;
    type Error = ();
    fn poll(&mut self) -> Poll<LaunchResult, ()> {
        match tokio_threadpool::blocking(|| {
            nix::sys::wait::waitpid(self.pid, None).expect("Failed to wait child process to exit")
        }) {
            Ok(Async::Ready(status)) => {
                let res = LaunchResult {
                    exit_success: match status {
                        nix::sys::wait::WaitStatus::Exited(_, code) => code == 0,
                        nix::sys::wait::WaitStatus::Signaled(_, _, _) => false,
                        _ => unreachable!("Should not appear other cases"),
                    },
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
