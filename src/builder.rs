use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::process::ChildExt;
use crate::workspace::{config::Builder as BuilderConfig, BuildDir};

pub struct Builder {
    build_dir: PathBuf,
    script: PathBuf,
    source_file: PathBuf,
    executable_file: PathBuf,
    target_dir: PathBuf,
    timeout: Option<Duration>,
}

impl Builder {
    pub fn new(build_dir: BuildDir, config: BuilderConfig) -> io::Result<Option<Builder>> {
        let script = {
            let res = if let Some(script) = config.build_script {
                Some(script)
            } else if let Some(language) = config.language {
                get_build_script_from_language(&language)
            } else if let Some(ext) = config.source.extension() {
                get_build_script_from_suffix(&ext)
            } else {
                None
            };
            match res {
                None => return Ok(None),
                Some(script) => script,
            }
        };

        Ok(Some(Builder {
            build_dir: build_dir.clone(),
            script,
            source_file: config.source,
            executable_file: build_dir.executable_file(),
            target_dir: build_dir.target_dir(),
            timeout: config.timeout.map(Duration::from_secs_f64),
        }))
    }

    pub async fn build(&self) -> io::Result<BuilderOutput> {
        let res = Command::new("/bin/sh")
            .arg("-c")
            .arg(&self.script)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env_clear()
            .env("SOURCE_FILE", &self.source_file)
            .env("EXECUTABLE_FILE", &self.executable_file)
            .env("TARGET_DIR", &self.target_dir)
            .current_dir(&self.build_dir)
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

fn get_build_script_from_language<P: AsRef<Path>>(language: P) -> Option<PathBuf> {
    let language_dir = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/lang")).join(language);
    if language_dir.exists() {
        Some(language_dir.join("build.sh"))
    } else {
        None
    }
}

fn get_build_script_from_suffix<P: AsRef<Path>>(suffix: P) -> Option<PathBuf> {
    let language = suffix.as_ref().with_extension(".default");
    get_build_script_from_language(&language)
}
