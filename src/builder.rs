use std::fs::{self, Permissions};
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::language::Language;
use crate::process::ChildExt;
use crate::workspace::BuildDir;
use crate::process::cgroup;
use crate::process::cgroup::CommandExt;

#[derive(Debug)]
pub struct Builder {
    build_dir: PathBuf,
    script: PathBuf,
    source_file: PathBuf,
    executable_file: PathBuf,
    target_dir: PathBuf,
    timeout: Option<Duration>,
}

impl Builder {
    pub fn new(build_dir: &BuildDir) -> io::Result<Option<Builder>> {
        let script = {
            let script = if let Some(ref script) = build_dir.config().build_script {
                Some(build_dir.join(script))
            } else if let Some(ref language) = build_dir.config().language {
                Language::new(language).map(|lang| lang.build_script())
            } else if let Some(ext) = build_dir.config().source.extension() {
                Language::from_ext(ext).map(|lang| lang.build_script())
            } else {
                None
            };
            match script {
                Some(script) => script,
                None => return Ok(None),
            }
        };
        let script = script.canonicalize()?;
        if !script.starts_with(&build_dir) {
            fs::copy(&script, &build_dir.build_script())?;
        }

        Ok(Some(Builder {
            build_dir: build_dir.as_path().to_owned(),
            script,
            source_file: build_dir.config().source.clone(),
            executable_file: build_dir.executable_file(),
            target_dir: build_dir.target_dir(),
            timeout: build_dir.config().timeout,
        }))
    }

    pub async fn build(&self) -> io::Result<BuilderOutput> {
        if !self.target_dir.exists() {
            fs::create_dir(&self.target_dir)?;
        }
        fs::set_permissions(&self.script, Permissions::from_mode(0o700))?;
        let cg_ctx = cgroup::Builder::new()
            .cpu_controller(true)
            .cpuacct_controller(true)
            .memory_controller(true)
            .cpuset_controller(true, 1)
            .build()
            .await?;
        #[warn(unused_variables)]
        let cg_holder = cgroup::ContextHolder {
            cg: cg_ctx.clone(),
        };
        let res = Command::new("/bin/sh")
            .arg("-c")
            .arg(&self.script)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env_clear()
            .env("PATH", "/usr/local/sbin:/usr/local/bin:/usr/bin:/bin")
            .env("SOURCE_FILE", &self.source_file)
            .env("EXECUTABLE_FILE", &self.executable_file)
            .env("TARGET_DIR", &self.target_dir)
            .current_dir(&self.build_dir)
            .cgroup(cg_ctx)
            .spawn()?;
        let res = match self.timeout {
            Some(timeout) => res.timeout_with_output(timeout)?,
            None => res.wait_with_output()?,
        };
        Ok(BuilderOutput {
            success: res.status.success(),
            stdout: res.stdout,
            stderr: res.stderr,
        })
    }
}

pub struct BuilderOutput {
    pub success: bool,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
