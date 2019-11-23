//! Work directory for judging process.
use std::io;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use config::Config;
use tempfile::TempDir;
use tokio::fs;
use tokio::fs::os::unix as unix_fs;

pub struct Workspace {
    inner: PathBuf,
}

impl Workspace {
    pub async fn new() -> io::Result<Workspace> {
        let res = Workspace {
            inner: TempDir::new()?.into_path(),
        };
        res.init().await?;
        Ok(res)
    }

    pub async fn new_in<P: AsRef<Path>>(dir: P) -> io::Result<Workspace> {
        let res = Workspace {
            inner: TempDir::new_in(dir)?.into_path(),
        };
        res.init().await?;
        Ok(res)
    }

    async fn init(&self) -> io::Result<()> {
        // FIXME: Use futures::join!()
        fs::create_dir_all(self.build_dir()).await?;
        fs::create_dir_all(self.build_dir().target_dir()).await?;
        fs::create_dir_all(self.runtime_dir()).await?;
        fs::create_dir_all(self.problem_dir()).await?;
        Ok(())
    }
}

impl Workspace {
    pub fn as_path(&self) -> &Path {
        self.inner.as_path()
    }

    pub async fn read_config(&self) -> io::Result<Config> {
        let config = fs::read(&self.config_file()).await?;
        Ok(toml::from_slice(&config)?)
    }

    pub fn config_file(&self) -> PathBuf {
        self.as_path().join("config.toml")
    }

    pub fn build_dir(&self) -> BuildDir {
        BuildDir::from(self.as_path().join("build"))
    }

    pub fn runtime_dir(&self) -> RuntimeDir {
        RuntimeDir::from(self.as_path().join("runtime"))
    }

    pub fn problem_dir(&self) -> ProblemDir {
        ProblemDir::from(self.as_path().join("problem"))
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        use std::fs;
        match fs::remove_dir(self.as_path()) {
            Ok(()) => {}
            Err(e) => eprintln!("Failed to remove workspace. {}", e),
        }
    }
}

pub struct BuildDir {
    inner: PathBuf,
}

impl BuildDir {
    pub async fn read_config(&self) -> io::Result<config::Builder> {
        let config_file = self.join("config.toml");
        let config = fs::read(config_file).await?;
        match toml::from_slice(&config) {
            Ok(res) => Ok(res),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }

    pub fn build_script(&self) -> PathBuf {
        self.join("build.sh")
    }

    pub fn target_dir(&self) -> PathBuf {
        self.join("target")
    }

    pub fn executable_file(&self) -> PathBuf {
        self.target_dir().join("main")
    }
}

impl From<PathBuf> for BuildDir {
    fn from(inner: PathBuf) -> BuildDir {
        BuildDir { inner }
    }
}

impl Deref for BuildDir {
    type Target = PathBuf;

    fn deref(&self) -> &PathBuf {
        &self.inner
    }
}

impl AsRef<Path> for BuildDir {
    fn as_ref(&self) -> &Path {
        self.inner.as_path()
    }
}

pub struct RuntimeDir {
    inner: PathBuf,
}

impl RuntimeDir {
    pub fn read_config(&self) -> config::Runner {
        unimplemented!()
    }

    pub fn executable_file(&self) -> PathBuf {
        self.join("main")
    }

    pub fn input_file(&self) -> PathBuf {
        self.join("input")
    }

    pub fn output_file(&self) -> PathBuf {
        self.join("output")
    }

    pub async fn activate_case(&self, case: &Case) -> io::Result<()> {
        unix_fs::symlink(case.input_file(), self.input_file()).await?;
        Ok(())
    }
}

impl From<PathBuf> for RuntimeDir {
    fn from(inner: PathBuf) -> RuntimeDir {
        RuntimeDir { inner }
    }
}

impl Deref for RuntimeDir {
    type Target = PathBuf;

    fn deref(&self) -> &PathBuf {
        &self.inner
    }
}

impl AsRef<Path> for RuntimeDir {
    fn as_ref(&self) -> &Path {
        self.inner.as_path()
    }
}

#[derive(Clone)]
pub struct ProblemDir {
    inner: PathBuf,
}

impl ProblemDir {
    pub fn read_config(&self) -> config::Problem {
        unimplemented!()
    }

    pub fn cases(&self) -> Cases {
        Cases::new(&self)
    }
}

impl From<PathBuf> for ProblemDir {
    fn from(inner: PathBuf) -> ProblemDir {
        ProblemDir { inner }
    }
}

impl Deref for ProblemDir {
    type Target = PathBuf;

    fn deref(&self) -> &PathBuf {
        &self.inner
    }
}

impl AsRef<Path> for ProblemDir {
    fn as_ref(&self) -> &Path {
        self.inner.as_path()
    }
}

pub struct Cases {
    problem_dir: ProblemDir,
    index: usize,
}

impl Cases {
    fn new(problem_dir: &ProblemDir) -> Cases {
        Cases {
            problem_dir: problem_dir.clone(),
            index: 0,
        }
    }
}

impl Iterator for Cases {
    type Item = Case;

    fn next(&mut self) -> Option<Case> {
        let case = self.problem_dir.join(self.index.to_string());
        if case.exists() {
            self.index += 1;
            return Some(Case(case));
        }
        None
    }
}

pub struct Case(PathBuf);

impl Case {
    pub fn input_file(&self) -> PathBuf {
        self.join("input")
    }

    pub fn answer_file(&self) -> PathBuf {
        self.join("answer")
    }
}

impl Deref for Case {
    type Target = PathBuf;

    fn deref(&self) -> &PathBuf {
        &self.0
    }
}

pub mod config {
    use std::path::PathBuf;

    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Config {
        pub problem: Problem,
        pub builder: Builder,
        pub runner: Option<Runner>,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Problem {
        pub r#type: ProblemType,
        pub limit: ResourceLimit,
        pub spj: Option<SpecialJudge>,
        pub ignore_white_space_at_eol: Option<bool>,
        pub ignore_empty_line_at_eof: Option<bool>,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct ResourceLimit {
        pub cpu_time: u64,
        pub real_time: u64,
        pub memory: usize,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub enum ProblemType {
        Normal,
        SpecialJudge,
        Interactive,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct SpecialJudge {
        pub source: PathBuf,
        pub language: Option<String>,
        pub build_script: Option<PathBuf>,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Builder {
        pub source: PathBuf,
        pub language: Option<String>,
        pub build_script: Option<PathBuf>,
        pub timeout: Option<f64>,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Runner {
        pub command: Option<PathBuf>,
        pub args: Option<Vec<String>>,
        pub cgroups: Option<Cgroups>,
    }

    impl Default for Runner {
        fn default() -> Runner {
            Runner {
                command: None,
                args: Some(Vec::new()),
                cgroups: None,
            }
        }
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Cgroups {}

    impl Default for Cgroups {
        fn default() -> Cgroups {
            Cgroups {}
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use std::fs;
        use std::io;
        use std::path::Path;

        use toml;

        fn read_config_file(file: impl AsRef<Path>) -> io::Result<Config> {
            match toml::from_slice(&fs::read(&file)?) {
                Ok(res) => Ok(res),
                Err(_) => Err(io::Error::from(io::ErrorKind::InvalidData)),
            }
        }

        #[test]
        fn test_normal_c() -> io::Result<()> {
            let _config = read_config_file("examples/workspace/normal_c/config.toml")?;
            Ok(())
        }

        #[test]
        fn test_spj_c() -> io::Result<()> {
            let _config = read_config_file("examples/workspace/spj_c/config.toml")?;
            Ok(())
        }

        #[test]
        fn test_custom_script() -> io::Result<()> {
            let _config = read_config_file("examples/workspace/custom_script/config.toml")?;
            Ok(())
        }
    }
}
