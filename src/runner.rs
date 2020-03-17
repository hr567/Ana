use std::ffi::OsString;
use std::io;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use crate::process::cgroup;
use crate::workspace::{RunnerConfig, RuntimeDir};

pub struct Runner {
    inner: Command,
    cg: cgroup::Context,
}

impl Runner {
    pub fn new(runtime_dir: &RuntimeDir, config: &RunnerConfig) -> io::Result<Runner> {
        let mut command = Command::new(match &config.command {
            Some(executable) => executable.clone(),
            None => runtime_dir.join("main"),
        });
        let args: Vec<_> = config
            .args
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|s| arguments_map(s.clone(), &runtime_dir))
            .collect();
        command.args(args).env_clear().current_dir(&runtime_dir);

        // TODO: handle cgroup configurations
        // let cgroups_config = config.cgroups.unwrap_or_default();
        let cgroups_context = cgroup::Builder::new()
            .cpu_controller(false)
            .cpuacct_controller(false)
            .memory_controller(false)
            .build()?;

        dbg!(&cgroups_context);
        let res = Runner {
            inner: command,
            cg: cgroups_context,
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
        Ok(Program::new(child, self.cg.clone()))
    }
}

fn arguments_map(arg: String, runtime_dir: &RuntimeDir) -> OsString {
    match arg.as_str() {
        "$EXECUTABLE_FILE" => PathBuf::from("/main").into_os_string(),
        "$INPUT_FILE" => runtime_dir.input_file().into_os_string(),
        "$OUTPUT_FILE" => runtime_dir.output_file().into_os_string(),
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
