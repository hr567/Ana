//! High-level APIs for cgroup (Linux control group).
mod attr_file;
mod controller;
mod hierarchy;

use std::fs::remove_dir;
use std::io;
use std::os::unix::process::CommandExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use std::sync::Arc;

use nix::unistd::Pid;
use rand;

pub use attr_file::AttrFile;
pub use controller::*;
pub use hierarchy::*;

const CGROUP_ROOT: &str = "/sys/fs/cgroup";

/// Cgroup context.
#[derive(Debug, Clone)]
pub struct Context {
    name: String,
    cpu_controller_enable: bool,
    cpuacct_controller_enable: bool,
    cpuset_controller_enable: bool,
    memory_controller_enable: bool,
    // cpuset_controller need to be holded by Context, which maintain the allocated cpu
    cpuset_data: Option<Arc<CpusetData>>,
}

impl Context {
    /// Get the cpu controller.
    ///
    /// Return `None` if the controller has not been initialized.
    pub fn cpu_controller(&self) -> Option<CpuController<PathBuf>> {
        if self.cpu_controller_enable {
            Some(CpuController::from_ctx(&self))
        } else {
            None
        }
    }

    /// Get the cpuacct controller.
    ///
    /// Return `None` if the controller has not been initialized.
    pub fn cpuacct_controller(&self) -> Option<CpuAcctController<PathBuf>> {
        if self.cpuacct_controller_enable {
            Some(CpuAcctController::from_ctx(&self))
        } else {
            None
        }
    }

    /// Get the cpuset controller.
    ///
    /// Return `None` if the controller has not been initialized. 
    pub fn cpuset_controller(&self) -> Option<CpusetController<PathBuf>> {
        if self.cpuset_controller_enable {
            Some(CpusetController::from_ctx(&self))
        } else {
            None
        }
    }

    /// Get the cpuacct controller.
    ///
    /// Return `None` if the controller has not been initialized.
    pub fn memory_controller(&self) -> Option<MemoryController<PathBuf>> {
        if self.memory_controller_enable {
            Some(MemoryController::from_ctx(&self))
        } else {
            None
        }
    }

    /// Add a process to the context.
    pub fn add_process(&mut self, pid: Pid) -> io::Result<()> {
        for hierarchy in self.hierarchies() {
            hierarchy.procs().write(&pid)?;
        }
        Ok(())
    }

    /// Add a task(thread) to the context.
    pub fn add_task(&mut self, pid: Pid) -> io::Result<()> {
        for hierarchy in self.hierarchies() {
            hierarchy.tasks().write(&pid)?;
        }
        Ok(())
    }

    pub unsafe fn remove(&self) -> io::Result<()> {
        for hierarchy in self.hierarchies() {
            remove_dir(hierarchy.path())?;
        }
        Ok(())
    }
}

impl Context {
    /// Root path of the cgroup filesystem.
    fn root() -> &'static Path {
        Path::new(CGROUP_ROOT)
    }

    /// All hierarchies that this cgroup context contains.
    fn hierarchies<'a>(&'a self) -> Vec<Box<dyn 'a + Hierarchy>> {
        let mut res: Vec<Box<dyn Hierarchy>> = Vec::new();
        if let Some(controller) = self.cpu_controller() {
            res.push(Box::new(controller));
        }
        if let Some(controller) = self.cpuacct_controller() {
            res.push(Box::new(controller));
        }
        if let Some(controller) = self.memory_controller() {
            res.push(Box::new(controller));
        }
        if let Some(controller) = self.cpuset_controller() {
            res.push(Box::new(controller));
        }
        res
    }
}

/// Cgroup Context Holder: hold the context lifetime
pub struct ContextHolder {
    pub cg: Context,
}

impl Drop for ContextHolder {
    fn drop(&mut self) {
        unsafe {
            if let Err(e) = self.cg.remove() {
                log::debug!("Error when dropping cgroup {}", e);
            }
        }
    }
}

/// Cgroup context builder.
pub struct Builder {
    name: Option<String>,
    cpu_controller: bool,
    cpuacct_controller: bool,
    memory_controller: bool,
    cpuset_controller: bool,
    cpuset_num: u32,
}

impl Builder {
    pub fn new() -> Builder {
        Default::default()
    }

    pub fn name(mut self, name: &str) -> Builder {
        self.name = Some(name.to_owned());
        self
    }

    pub fn cpu_controller(mut self, flag: bool) -> Builder {
        self.cpu_controller = flag;
        self
    }

    pub fn cpuacct_controller(mut self, flag: bool) -> Builder {
        self.cpuacct_controller = flag;
        self
    }
    
    // TODO: implement cpuset allocate by numa align
    pub fn cpuset_controller(mut self, flag: bool, num_of_cpu: u32) -> Builder {
        self.cpuset_controller = flag;
        self.cpuset_num = num_of_cpu;
        self
    }

    pub fn memory_controller(mut self, flag: bool) -> Builder {
        self.memory_controller = flag;
        self
    }

    pub async fn build(self) -> io::Result<Context> {
        let name = match self.name {
            Some(name) => name,
            None => {
                let timestamp = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as u64;
                let salt: u64 = rand::random();
                format!("{:x}{:x}", timestamp, salt)
            }
        };

        let mut ctx = Context {
            name,
            cpu_controller_enable: self.cpu_controller,
            cpuacct_controller_enable: self.cpuacct_controller,
            cpuset_controller_enable: self.cpuset_controller,
            memory_controller_enable: self.memory_controller,
            cpuset_data: None,
        };

        if self.cpu_controller {
            let controller = CpuController::from_ctx(&ctx);
            controller.initialize()?
        }

        if self.cpuacct_controller {
            let controller = CpuAcctController::from_ctx(&ctx);
            controller.initialize()?
        }

        if self.memory_controller {
            let controller = MemoryController::from_ctx(&ctx);
            controller.initialize()?
        }

        if self.cpuset_controller {
            let mut controller = CpusetController::from_ctx(&ctx);
            controller.initialize()?;
            ctx.cpuset_data = Some(
                controller.allocate(self.cpuset_num).await?
            );
        }

        Ok(ctx)
    }
}

impl Default for Builder {
    fn default() -> Builder {
        Builder {
            name: None,
            cpu_controller: true,
            cpuacct_controller: true,
            memory_controller: true,
            cpuset_controller: false,
            cpuset_num: 0
        }
    }
}

pub trait CommandExt {
    /// Attach the child process to the cgroup.
    fn cgroup(&mut self, ctx: Context) -> &mut Command;
}

impl CommandExt for Command {
    fn cgroup(&mut self, mut ctx: Context) -> &mut Command {
        // Ensure that the cgroup context will not be dropped in child process
        unsafe {
            self.pre_exec(move || {
                ctx.add_process(nix::unistd::Pid::this())?;
                Ok(())
            });
        }
        self
    }
}

#[cfg(test)]
mod tests;
