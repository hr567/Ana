use std::fs;
use std::io;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use nix;
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

pub struct RuntimeHolder {
    runtime_dir: PathBuf,
    work_dir: Option<PathBuf>,
    upper_dir: Option<PathBuf>,
    with_rootfs: bool,
}

impl RuntimeHolder {
    pub fn new(
        runtime_dir: &RuntimeDir,
        rootfs_config: Option<&RootfsConfig>,
    ) -> io::Result<RuntimeHolder> {
        let mut with_rootfs = false;
        let mut work_dir = None;
        let mut upper_dir = None;
        let runtime_dir = runtime_dir.inner.clone();

        fs::create_dir_all(&runtime_dir)?;

        if let Some(config) = rootfs_config {
            with_rootfs = true;
            work_dir = Some(
                runtime_dir
                    .parent()
                    .expect("runtime dir should not be /")
                    .join("work"),
            );
            upper_dir = Some(
                runtime_dir
                    .parent()
                    .expect("runtime dir should not be /")
                    .join("upper"),
            );

            fs::create_dir_all(&work_dir.as_ref().unwrap())?;
            fs::create_dir_all(&upper_dir.as_ref().unwrap())?;

            let data = format!(
                "lowerdir={},upperdir={},workdir={}",
                config.base_path.display(),
                work_dir.as_ref().unwrap().display(),
                upper_dir.as_ref().unwrap().display()
            );

            nix::mount::mount(
                Some("overlay"),
                &runtime_dir,
                Some("overlay"),
                nix::mount::MsFlags::empty(),
                Some(data.as_str()),
            )
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "mount runtime overlay failed!"))?;
        }

        Ok(RuntimeHolder {
            runtime_dir: runtime_dir,
            work_dir: work_dir,
            upper_dir: upper_dir,
            with_rootfs: with_rootfs,
        })
    }
}

impl Drop for RuntimeHolder {
    fn drop(&mut self) {
        log::debug!("droping runtime dir");
        if self.with_rootfs {
            if let Err(e) = nix::mount::umount(&self.runtime_dir) {
                log::debug!(
                    "Error when umount runntime_dir {}, err: {}",
                    self.runtime_dir.display(),
                    e
                );
            }
            let work_dir = self.work_dir.as_ref().unwrap();
            let upper_dir = self.upper_dir.as_ref().unwrap();

            if let Err(e) = fs::remove_dir_all(work_dir) {
                log::debug!(
                    "Error when remove work dir {}, err: {}",
                    work_dir.display(),
                    e
                );
            }
            if let Err(e) = fs::remove_dir_all(upper_dir) {
                log::debug!(
                    "Error when remove upper dir {}, err: {}",
                    upper_dir.display(),
                    e
                );
            }
        }

        if let Err(e) = fs::remove_dir_all(&self.runtime_dir) {
            log::debug!(
                "Error when remove runtime dir {}, err: {}",
                self.runtime_dir.display(),
                e
            );
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RunnerConfig {
    pub command: Option<PathBuf>,
    pub args: Option<Vec<String>>,
    pub cgroups: Option<CgroupsConfig>,
    pub seccomp: Option<SeccompConfig>,
    pub namespaces: Option<Vec<Namespace>>,
    pub rootfs: Option<RootfsConfig>,
    pub envs: Option<BTreeMap<String, String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RootfsConfig {
    pub base_path: PathBuf,
    pub with_proc: bool,
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
