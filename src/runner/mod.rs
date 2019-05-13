/// Program runner with resource limit
/// and system call filter
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

mod cgroup;
mod seccomp;

/// Report the runner return
#[derive(Debug)]
pub struct RunnerReport {
    /// set to `true` if the program exit with code zero and
    /// set to `false` if exited with non-zero code or be killed
    pub exit_success: bool,

    /// time the program use in real world
    pub real_time_usage: time::Duration,

    /// cpu time the program use
    pub cpu_time_usage: time::Duration,

    /// memory the program use
    pub memory_usage: usize,

    /// set to `true` if the program timeout
    pub tle_flag: bool,

    /// set to `true` if the program use too many memory
    pub mle_flag: bool,
}

pub fn run(
    chroot_dir: Option<impl AsRef<path::Path>>,
    program: impl AsRef<path::Path>,
    input_file: impl AsRef<path::Path>,
    output_file: impl AsRef<path::Path>,
    time_limit: time::Duration,
    memory_limit: usize,
) -> RunnerReport {
    match nix::unistd::fork().expect("Failed to fork a child process") {
        nix::unistd::ForkResult::Parent { child: pid } => {
            let start_time = time::Instant::now();

            let cg = cgroup::Cgroup::new(time_limit, memory_limit);
            cg.add_process(pid);

            thread::spawn(move || {
                thread::sleep(time_limit * 2);
                nix::sys::signal::kill(pid, nix::sys::signal::SIGKILL).unwrap_or_default();
            });

            RunnerReport {
                exit_success: match nix::sys::wait::waitpid(pid, None)
                    .expect("Failed to wait child process to exit")
                {
                    nix::sys::wait::WaitStatus::Exited(_, code) => code == 0,
                    nix::sys::wait::WaitStatus::Signaled(_, _, _) => false,
                    _ => unreachable!("Should not appear other cases"),
                },
                real_time_usage: start_time.elapsed(),
                cpu_time_usage: cg.get_cpu_time_usage(),
                memory_usage: cg.get_memory_usage(),
                tle_flag: cg.is_time_limit_exceeded(),
                mle_flag: cg.is_memory_limit_exceeded(),
            }
        }

        nix::unistd::ForkResult::Child => {
            // Replace stdio with files
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

            // Unshare namespace
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

            // Chroot if needed
            if let Some(chroot_dir) = chroot_dir {
                nix::unistd::chroot(chroot_dir.as_ref()).expect("Failed to chroot");
            }

            // Change directory to root
            nix::unistd::chdir("/").expect("Failed to chdir");

            unsafe {
                nix::libc::prctl(nix::libc::PR_SET_NO_NEW_PRIVS, 1);
                nix::libc::prctl(nix::libc::PR_SET_DUMPABLE, 0);
            }

            // Clear all capabilities
            caps::clear(None, caps::CapSet::Permitted).expect("Failed to clear all capabilities");

            // Change program to C-style string
            let program_command =
                ffi::CString::new(program.as_ref().as_os_str().as_bytes()).unwrap();

            // Use seccomp to prevent some system calls
            let scmp = seccomp::ScmpCtx::new(); // Default rule is kill
            for syscall in &[
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
            ] {
                scmp.whitelist(*syscall as u32, Vec::new())
                    .expect("Failed to add syscall to whitelist");
            }

            // Only allow exec program
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

            nix::unistd::execvpe(&program_command, &[program_command.clone()], &[])
                .expect("Failed to exec child process");

            unreachable!("Not reachable after exec")
        }
    }
}
