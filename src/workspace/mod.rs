//! Work directory for judging process.
pub mod build;
pub mod problem;
pub mod runtime;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[cfg(not(test))]
use log;
use serde::{Deserialize, Serialize};

pub use {
    build::BuildDir,
    problem::ProblemDir,
    runtime::{RunnerConfig, RuntimeDir},
};

pub struct Workspace {
    inner: PathBuf,
    build_dir: BuildDir,
    problem_dir: ProblemDir,
    runtime_dir: RuntimeDir,
    config: Config,
}

impl Workspace {
    pub fn from_path<P: AsRef<Path>>(dir: P) -> io::Result<Workspace> {
        let dir = dir.as_ref();

        let build_dir = BuildDir::from_path(dir.join("build"))?;
        let runtime_dir = RuntimeDir::from_path(dir.join("runtime"));
        let problem_dir = ProblemDir::from_path(dir.join("problem"))?;

        let config_file = dir.join("config.toml");
        let toml_config = fs::read(config_file)?;
        let config: Config = toml::from_slice(&toml_config)?;

        let res = Workspace {
            inner: PathBuf::from(dir),
            build_dir,
            problem_dir,
            runtime_dir,
            config,
        };

        // TODO: Check if the given path is available
        Ok(res)
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}

impl Workspace {
    pub fn as_path(&self) -> &Path {
        self.inner.as_path()
    }

    pub fn build_dir(&self) -> &BuildDir {
        &self.build_dir
    }

    pub fn runtime_dir(&self) -> &RuntimeDir {
        &self.runtime_dir
    }

    pub fn problem_dir(&self) -> &ProblemDir {
        &self.problem_dir
    }
}

#[cfg(not(test))]
impl Drop for Workspace {
    fn drop(&mut self) {
        match fs::remove_dir_all(self.as_path()) {
            Ok(()) => {}
            Err(e) => log::warn!("Failed to remove workspace. {}", e),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub runner: RunnerConfig,
}

pub struct Builder {}

impl Builder {
    pub fn new() -> Builder {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests;
