use std::ffi::OsString;
use std::io;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio, ChildStderr};
use std::time::Duration;
use std::collections::BTreeMap;

use crate::process::{cgroup, cgroup::CommandExt as _, CommandExt as _};
use crate::workspace::{RunnerConfig, RuntimeDir};

pub struct Runner {
    inner: Command,
    cg: cgroup::Context,
    proc_path: Option<PathBuf>,
}

impl Runner {
    pub fn new(runtime_dir: &RuntimeDir, config: &RunnerConfig) -> io::Result<Runner> {
        let mut with_proc = false;
        let mut proc_path = None;

        let executable_file = match &config.command {
            Some(executable) => executable.clone(),
            None => PathBuf::from("/main"),
        };
        let mut command = Command::new(&executable_file);
        let args: Vec<_> = config
            .args
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|arg| match arg.as_str() {
                "$EXECUTABLE_FILE" => executable_file.clone().into_os_string(),
                "$INPUT_FILE" => runtime_dir.input_file().into_os_string(),
                "$OUTPUT_FILE" => runtime_dir.output_file().into_os_string(),
                _ => OsString::from(arg),
            })
            .collect();
        let empty_envs = BTreeMap::new();
        command
            .args(args)
            .env_clear()
            .envs(config.envs.as_ref().unwrap_or(&empty_envs))
            .current_dir(&runtime_dir);

        if let Some(rootfs_config) = config.rootfs.as_ref() {
            with_proc = rootfs_config.with_proc;
        }


        // TODO: handle cgroup configurations
        // let cgroups_config = config.cgroups.unwrap_or_default();
        let cgroups_context = cgroup::Builder::new()
            .cpu_controller(true)
            .cpuacct_controller(true)
            .memory_controller(true)
            .build()?;
        command.cgroup(cgroups_context.clone());
        command.unshare_all_ns();
        command.chroot(runtime_dir);

        if with_proc {
            command.with_proc();
            proc_path = Some(runtime_dir.join("proc"));
        }

        dbg!(&cgroups_context);
        let res = Runner {
            inner: command,
            cg: cgroups_context,
            proc_path: proc_path
        };

        Ok(res)
    }

    pub fn stdin<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Runner {
        self.inner.stdin(cfg);
        self
    }

    pub fn stdout<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Runner {
        self.inner.stdout(cfg);
        self
    }

    pub fn stderr<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Runner {
        self.inner.stderr(cfg);
        self
    }

    pub fn spawn(&mut self) -> io::Result<Program> {
        let child = self.inner.spawn()?;
        Ok(Program::new(child, self.cg.clone(), self.proc_path.clone()))
    }
}

pub struct Program {
    inner: Child,
    cg: cgroup::Context,
    proc_path: Option<PathBuf>,
}

impl Program {
    fn new(inner: Child, cg: cgroup::Context, proc_path: Option<PathBuf>) -> Program {
        Program { inner, cg, proc_path }
    }

    pub fn get_resource_usage(&self) -> io::Result<(usize, Duration)> {
        let res = (
            match self.cg.memory_controller() {
                Some(controller) => controller.max_usage_in_bytes()?,
                None => 0,
            },
            match self.cg.cpuacct_controller() {
                Some(controller) => controller.usage()?,
                None => Duration::from_secs(0),
            },
        );
        Ok(res)
    }

    pub fn stderr(&mut self) -> Option<&mut ChildStderr> {
        self.inner.stderr.as_mut()
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            if let Err(e) = self.cg.remove() {
                log::debug!("Error when dropping cgroup {}", e);
            }

            if let Some(path) = self.proc_path.as_ref() {
                if let Err(e) = nix::mount::umount(path) {
                    log::debug!("Error when umount proc filesystem {}, {}", path.display(), e);
                }
            }
        }
    }
}

impl Deref for Program {
    type Target = Child;

    fn deref(&self) -> &Child {
        &self.inner
    }
}

impl DerefMut for Program {
    fn deref_mut(&mut self) -> &mut Child {
        &mut self.inner
    }
}
