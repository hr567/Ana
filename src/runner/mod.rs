use std::ffi;
use std::fs;
use std::io;
use std::path;
use std::thread;
use std::time;

use std::os::unix::{
    ffi::OsStrExt as _,
    io::{AsRawFd as _, IntoRawFd as _},
};

use caps;
use nix;
use tokio::prelude::*;
use tokio_threadpool;

mod cgroup;
mod seccomp;

#[derive(Debug)]
pub struct RunnerReport {
    pub exit_success: bool,
    pub real_time_usage: time::Duration,
    pub cpu_time_usage: time::Duration,
    pub memory_usage: usize,
    pub tle_flag: bool,
    pub mle_flag: bool,
}

/// Use input file to replace stdin and
/// use output file to replace stdout
fn replace_stdio(input_file: impl AsRef<path::Path>, output_file: impl AsRef<path::Path>) {
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
}

/// Unshare some namespaces
fn unshare_namespace() {
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
}

pub fn run(
    chroot_dir: Option<impl AsRef<path::Path>>,
    program: impl AsRef<path::Path>,
    input_file: impl AsRef<path::Path>,
    output_file: impl AsRef<path::Path>,
    time_limit: time::Duration,
    memory_limit: usize,
) -> impl Future<Item = RunnerReport, Error = ()> {
    let cg = cgroup::Cgroup::new(time_limit, memory_limit);

    match nix::unistd::fork().expect("Failed to fork a child process") {
        nix::unistd::ForkResult::Parent { child: pid } => {
            cg.add_process(pid);

            thread::spawn(move || {
                let start_time = time::Instant::now();
                while start_time.elapsed() < time_limit * 2 {
                    thread::sleep(time_limit / 2);
                }
                nix::sys::signal::kill(pid, nix::sys::signal::SIGKILL).unwrap_or_else(|e| {
                    assert_eq!(
                        e,
                        nix::Error::Sys(nix::errno::Errno::ESRCH),
                        "Failed to kill long running child process"
                    );
                });
            });

            RunnerFuture {
                cg,
                pid,
                start_time: time::Instant::now(),
            }
        }

        nix::unistd::ForkResult::Child => {
            let program_command =
                ffi::CString::new(program.as_ref().as_os_str().as_bytes()).unwrap();

            replace_stdio(input_file, output_file);
            unshare_namespace();

            // Chroot and chdir
            if let Some(chroot_dir) = chroot_dir {
                nix::unistd::chroot(chroot_dir.as_ref()).expect("Failed to chroot");
            }
            nix::unistd::chdir("/").expect("Failed to chdir");

            unsafe {
                nix::libc::prctl(nix::libc::PR_SET_NO_NEW_PRIVS, 1);
                nix::libc::prctl(nix::libc::PR_SET_DUMPABLE, 0);
            }

            caps::clear(None, caps::CapSet::Permitted).expect("Failed to clear all capabilities");

            {
                let whitelist: [i32; 16] = [
                    seccomp::syscall("access"),
                    seccomp::syscall("arch_prctl"),
                    seccomp::syscall("brk"),
                    seccomp::syscall("close"),
                    seccomp::syscall("exit_group"),
                    seccomp::syscall("fstat"),
                    seccomp::syscall("lseek"),
                    seccomp::syscall("mmap"),
                    seccomp::syscall("mprotect"),
                    seccomp::syscall("munmap"),
                    seccomp::syscall("read"),
                    seccomp::syscall("readlink"),
                    seccomp::syscall("sysinfo"),
                    seccomp::syscall("uname"),
                    seccomp::syscall("write"),
                    seccomp::syscall("writev"),
                ];

                let scmp = seccomp::ScmpCtx::new();
                for syscall in &whitelist {
                    scmp.whitelist(*syscall as u32, Vec::new())
                        .expect("Failed to add syscall to whitelist");
                }
                scmp.whitelist(
                    seccomp::syscall("execve") as u32,
                    vec![seccomp::ScmpArg::new(
                        0,
                        seccomp::ScmpCmp::EQ,
                        program_command.as_ptr() as u64,
                    )],
                )
                .expect("Failed to add execve to whitelist");
                scmp.load().expect("Failed to set seccomp");
            }

            nix::unistd::execvpe(&program_command, &[program_command.clone()], &[])
                .expect("Failed to exec child process");

            unreachable!("Not reachable after exec")
        }
    }
}

pub struct RunnerFuture {
    cg: cgroup::Cgroup,
    pid: nix::unistd::Pid,
    start_time: time::Instant,
}

unsafe impl Send for RunnerFuture {}

impl Future for RunnerFuture {
    type Item = RunnerReport;
    type Error = ();
    fn poll(&mut self) -> Poll<RunnerReport, ()> {
        match tokio_threadpool::blocking(|| {
            nix::sys::wait::waitpid(self.pid, None).expect("Failed to wait child process to exit")
        }) {
            Ok(Async::Ready(status)) => {
                let res = RunnerReport {
                    exit_success: match status {
                        nix::sys::wait::WaitStatus::Exited(_, code) => code == 0,
                        nix::sys::wait::WaitStatus::Signaled(_, _, _) => false,
                        _ => unreachable!("Should not appear other cases"),
                    },
                    real_time_usage: self.start_time.elapsed(),
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
