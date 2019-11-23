use std::ffi::OsString;
use std::io;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::time::Duration;

use crate::process::{cgroup, ChildExt, CommandExt};
use crate::workspace::{config::Runner as RunnerConfig, RuntimeDir};

pub struct Runner {
    inner: Command,
    cg: cgroup::Context,
}

impl Runner {
    pub fn new(runtime_dir: &RuntimeDir, config: RunnerConfig) -> io::Result<Runner> {
        let mut command = Command::new(match config.command {
            Some(executable) => executable,
            None => runtime_dir.executable_file(),
        });
        let args = config
            .args
            .unwrap_or_default()
            .into_iter()
            .map(|s| arguments_map(s, &runtime_dir));
        command.args(args).env_clear().current_dir(&runtime_dir);

        let _cgroups_config = config.cgroups.unwrap_or_default();

        let res = Runner {
            inner: command,
            cg: unimplemented!("TODO: cgroups setting"),
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
        Ok(Program {
            inner: child,
            cg: self.cg.clone(),
        })
    }
}

fn arguments_map(arg: String, runtime_dir: &RuntimeDir) -> OsString {
    match arg.as_str() {
        "$EXECUTABLE_FILE" => runtime_dir.executable_file().into_os_string(),
        "$INPUT_FILE" => runtime_dir.join("input").into_os_string(),
        "$OUTPUT_FILE" => runtime_dir.join("output").into_os_string(),
        _ => OsString::from(arg),
    }
}

pub struct Program {
    inner: Child,
    cg: cgroup::Context,
}

impl Program {
    fn new(inner: Child, cg: cgroup::Context) -> Program {
        Program { inner, cg }
    }

    pub fn get_resource_usage(&self) -> io::Result<(usize, Duration)> {
        Ok((
            match self.cg.memory_controller() {
                Some(controller) => controller.max_usage_in_bytes()?,
                None => 0,
            },
            match self.cg.cpuacct_controller() {
                Some(controller) => controller.usage()?,
                None => Duration::from_secs(0),
            },
        ))
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
