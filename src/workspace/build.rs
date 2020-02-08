use std::fs;
use std::io;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use toml;

#[derive(Clone)]
pub struct BuildDir {
    inner: PathBuf,
    config: Config,
}

impl BuildDir {
    pub fn from_path<P: AsRef<Path>>(inner: P) -> io::Result<BuildDir> {
        let inner = inner.as_ref();
        let config_file = inner.join("config.toml");
        let toml_config = fs::read(config_file)?;
        let config = toml::from_slice(&toml_config)?;
        Ok(BuildDir {
            inner: PathBuf::from(inner),
            config,
        })
    }

    pub fn build_script(&self) -> PathBuf {
        match &self.config.build_script {
            Some(build_script) => self.join(build_script),
            None => self.join("build.sh"),
        }
    }

    pub fn target_dir(&self) -> PathBuf {
        self.join("target")
    }

    pub fn executable_file(&self) -> PathBuf {
        self.target_dir().join("main")
    }

    pub fn config(&self) -> &Config {
        &self.config
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub source: PathBuf,
    pub language: Option<String>,
    pub build_script: Option<PathBuf>,
    pub timeout: Option<Duration>,
}
