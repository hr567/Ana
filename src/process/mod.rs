// TODO: Add tests
//! Run a program in a container with resource limit and system calls filter.
#[cfg(feature = "cap-ng")]
pub mod capng;
pub mod cgroup;
#[cfg(feature = "seccomp")]
pub mod seccomp;

use std::io;
use std::os::unix::process::CommandExt as _;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Output};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use nix;

/// Extra features make `Command` run in a new container.
pub trait CommandExt {
    /// Run program with all namespaces unshared.
    fn unshare_all_ns(&mut self) -> &mut Command;

    /// Chroot to a new path before exec.
    fn chroot<P: AsRef<Path>>(&mut self, new_root: P) -> &mut Command;
}

impl CommandExt for Command {
    fn unshare_all_ns(&mut self) -> &mut Command {
        unsafe {
            self.pre_exec(move || {
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
                Ok(())
            });
        }
        self
    }

    fn chroot<P: AsRef<Path>>(&mut self, new_root: P) -> &mut Command {
        let new_root = new_root.as_ref().to_owned();
        unsafe {
            self.pre_exec(move || {
                nix::unistd::chroot(&new_root).expect("Failed to chroot to new path");
                nix::unistd::chdir("/").expect("Failed to chdir to new root");
                Ok(())
            });
        }
        self
    }
}

/// Extra features for child process.
pub trait ChildExt {
    /// Wait for the child process, returning the exit status.
    /// The child process will be killed if it waits more than `timeout`.
    fn timeout(&mut self, timeout: Duration) -> io::Result<ExitStatus>;
    /// Wait for the child process, returning the output of the child process.
    /// The child process will be killed if it waits more than `timeout`.
    fn timeout_with_output(self, timeout: Duration) -> io::Result<Output>;
}

impl ChildExt for Child {
    fn timeout(&mut self, timeout: Duration) -> io::Result<ExitStatus> {
        let (tx, rx) = mpsc::channel();
        let pid = nix::unistd::Pid::from_raw(self.id() as nix::libc::pid_t);
        thread::spawn(move || {
            match rx.recv_timeout(timeout) {
                // Do nothing if the child process exit before times out
                Ok(_) => {}
                // Kill the child process if it times out
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    let _ = nix::sys::signal::kill(pid, nix::sys::signal::SIGKILL);
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    panic!("Channel disconnected unexpected")
                }
            }
        });
        let status = self.wait()?;
        let _ = tx.send(());
        Ok(status)
    }

    fn timeout_with_output(self, timeout: Duration) -> io::Result<Output> {
        let (tx, rx) = mpsc::channel();
        let pid = nix::unistd::Pid::from_raw(self.id() as nix::libc::pid_t);
        thread::spawn(move || {
            match rx.recv_timeout(timeout) {
                Ok(_) => {} // Do nothing if the child process exit before times out
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Kill the child process if it times out
                    let _ = nix::sys::signal::kill(pid, nix::sys::signal::SIGKILL);
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    panic!("Channel disconnected unexpected")
                }
            }
        });
        let output = self.wait_with_output()?;
        let _ = tx.send(());
        Ok(output)
    }
}

#[cfg(test)]
mod tests;
