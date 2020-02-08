use std::ops::Deref;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub struct RuntimeDir {
    inner: PathBuf,
}

impl RuntimeDir {
    pub fn from_path<P: AsRef<Path>>(inner: P) -> RuntimeDir {
        let inner = inner.as_ref();
        RuntimeDir {
            inner: PathBuf::from(inner),
        }
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
}

impl From<&Path> for RuntimeDir {
    fn from(inner: &Path) -> RuntimeDir {
        RuntimeDir {
            inner: PathBuf::from(inner),
        }
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RunnerConfig {
    pub command: Option<PathBuf>,
    pub args: Option<Vec<String>>,
    pub cgroups: Option<CgroupsConfig>,
    pub seccomp: Option<SeccompConfig>,
    pub namespaces: Option<Vec<Namespace>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Namespace {
    CGROUP,
    IPC,
    NETWORK,
    MOUNT,
    PID,
    USER,
    UTS,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct CgroupsConfig {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SeccompConfig {}
