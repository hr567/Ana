use std::fs::{read_to_string, write};
use std::io;
use std::path::Path;

use nix::unistd::Pid;

use super::AttrFile;

/// Hierarchy in the cgroup.
pub trait Hierarchy<'a> {
    /// The path of this hierarchy.
    fn path(&self) -> &Path;

    /// `cgroup.procs` file in this hierarchy.
    ///
    /// Can read from and write to this it.
    fn procs(&self) -> Box<dyn AttrFile<'a, Pid, Vec<Pid>>>;

    /// `cgroup.tasks` file in this hierarchy.
    ///
    /// Can read from and write to this it.
    fn tasks(&self) -> Box<dyn AttrFile<'a, Pid, Vec<Pid>>>;
}

impl<'a, T: AsRef<Path>> Hierarchy<'a> for T {
    fn path(&self) -> &Path {
        self.as_ref()
    }

    fn procs(&self) -> Box<dyn AttrFile<'a, Pid, Vec<Pid>>> {
        Box::new(PidFile::from(self.path().join("cgroup.procs")))
    }

    fn tasks(&self) -> Box<dyn AttrFile<'a, Pid, Vec<Pid>>> {
        Box::new(PidFile::from(self.path().join("tasks")))
    }
}

struct PidFile<T: AsRef<Path>> {
    inner: T,
}

impl<T: AsRef<Path>> From<T> for PidFile<T> {
    fn from(inner: T) -> PidFile<T> {
        PidFile { inner }
    }
}

impl<'a, T: AsRef<Path>> AttrFile<'a, Pid, Vec<Pid>> for PidFile<T> {
    fn write(&mut self, pid: &Pid) -> io::Result<()> {
        write(&self.inner, pid.to_string())?;
        Ok(())
    }

    fn read(&self) -> io::Result<Vec<Pid>> {
        Ok(read_to_string(&self.inner)?
            .split_whitespace()
            .map(|pid| -> Pid {
                Pid::from_raw(
                    pid.trim()
                        .parse()
                        .expect("Failed to get pid from task file"),
                )
            })
            .collect())
    }
}
