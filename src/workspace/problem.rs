use std::fs;
use std::io;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::time::Duration;

use log;
use serde::{Deserialize, Serialize};
use toml;

#[derive(Clone)]
pub struct ProblemDir {
    inner: PathBuf,
    config: Config,
}

impl ProblemDir {
    pub fn from_path<P: AsRef<Path>>(inner: P) -> io::Result<ProblemDir> {
        let inner = inner.as_ref();
        let config_file = inner.join("config.toml");
        let toml_config = fs::read(config_file)?;
        let config: Config = toml::from_slice(&toml_config)?;
        match config.problem_type {
            ProblemType::Normal => {
                if config.extern_program.is_some() {
                    log::warn!(
                        "The problem in {:?} is a normal problem but the extern program is exist.",
                        inner,
                    );
                }
            }
            ProblemType::SpecialJudge => {
                if config.ignore_empty_line_at_eof.is_some() {
                    log::warn!(
                        "The problem in {:?} is a special judge problem but ignore empty line at end of file config is exist",
                        inner,
                    );
                }
                if config.ignore_white_space_at_eol.is_some() {
                    log::warn!(
                        "The problem in {:?} is a special judge problem but ignore white space at end of line config is exist.",
                        inner,
                    );
                }
            }
            ProblemType::Interactive => {
                if config.extern_program.is_some() {
                    log::warn!(
                        "The problem in {:?} is an interactive problem but the extern program is exist.",
                        inner,
                    );
                }
                if config.ignore_empty_line_at_eof.is_some() {
                    log::warn!(
                        "The problem in {:?} is an interactive problem but ignore empty line at end of file config is exist.",
                        inner,
                    );
                }
                if config.ignore_white_space_at_eol.is_some() {
                    log::warn!(
                        "The problem in {:?} is an interactive problem but ignore white space at end of line config is exist.",
                        inner,
                    );
                }
            }
        }
        Ok(ProblemDir {
            inner: PathBuf::from(inner),
            config,
        })
    }

    pub fn check_script(&self) -> PathBuf {
        self.join("check.sh")
    }

    pub fn extern_program(&self) -> PathBuf {
        self.join("extern_program")
    }

    pub fn cases(&self) -> Cases {
        Cases::new(&self)
    }

    pub fn config(&self) -> &Config {
        &self.config
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
    pub fn new(inner: PathBuf) -> Case {
        Case(inner)
    }

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

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub problem_type: ProblemType,
    pub limit: ResourceLimit,
    pub extern_program: Option<ExternProgram>,
    pub ignore_white_space_at_eol: Option<bool>,
    pub ignore_empty_line_at_eof: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ResourceLimit {
    pub cpu_time: Duration,
    pub real_time: Duration,
    pub memory: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ProblemType {
    Normal,
    SpecialJudge,
    Interactive,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ExternProgram {
    pub source: PathBuf,
    pub language: Option<String>,
    pub build_script: Option<PathBuf>,
}
