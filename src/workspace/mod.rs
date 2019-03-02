/// Workspace on tmpfs
///
/// Use fuse rewrite in the future for
/// better performance and less memory usage
use std::fs;
use std::ops;
use std::path;
use std::sync;

use libmount;
use log::*;
use tempfile;

use crate::mtp;

mod problem_dir;
mod source_dir;
mod test_case_dir;

pub use problem_dir::ProblemDir;
pub use source_dir::SourceDir;
pub use test_case_dir::TestCaseDir;

#[derive(Clone)]
pub struct WorkSpace {
    inner: sync::Arc<TempDir>,
}

impl WorkSpace {
    pub fn new() -> WorkSpace {
        let workspace = WorkSpace {
            inner: sync::Arc::new(TempDir(
                tempfile::tempdir().expect("Failed to create a temp dir"),
            )),
        };

        libmount::Tmpfs::new(&workspace)
            .mode(0o700)
            .mount()
            .expect("Failed to mount tmpfs on workspace");

        fs::create_dir(workspace.source_dir()).unwrap();
        fs::create_dir(workspace.problem_dir()).unwrap();
        fs::create_dir(workspace.runtime_dir()).unwrap();

        libmount::Tmpfs::new(&workspace.runtime_dir())
            .mount()
            .unwrap();

        debug!("Create new workspace in {:?}", &workspace.inner.0);
        workspace
    }

    fn join(&self, filename: &str) -> Box<path::Path> {
        self.inner.path().join(filename).into_boxed_path()
    }

    pub fn id_file(&self) -> Box<path::Path> {
        self.join("id")
    }

    pub fn runtime_dir(&self) -> Box<path::Path> {
        self.join("runtime")
    }

    pub fn source_dir(&self) -> Box<path::Path> {
        self.join("source")
    }

    pub fn problem_dir(&self) -> Box<path::Path> {
        self.join("problem")
    }

    pub fn remount_runtime_dir(&self) {
        libmount::Remount::new(&self.runtime_dir())
            .readonly(true)
            .remount()
            .expect("Failed to remount runtime directory");
    }
}

impl WorkSpace {
    pub fn get_id(&self) -> String {
        let id = fs::read(self.id_file()).unwrap();
        String::from_utf8(id).unwrap()
    }
}

impl WorkSpace {
    pub fn prepare_judge_task(&self, judge_task: mtp::JudgeTask) {
        let mtp::JudgeTask {
            id,
            source,
            problem,
        } = judge_task;
        fs::write(self.id_file(), id).unwrap();
        self.init_source_dir(source);
        self.init_problem_dir(problem);
    }
}

impl AsRef<path::Path> for WorkSpace {
    fn as_ref(&self) -> &path::Path {
        self.inner.0.path()
    }
}

struct TempDir(tempfile::TempDir);

impl Drop for TempDir {
    fn drop(&mut self) {
        nix::mount::umount(self.path().join("runtime").as_path()).unwrap();
        nix::mount::umount(self.path()).unwrap();
    }
}

impl ops::Deref for TempDir {
    type Target = tempfile::TempDir;

    fn deref(&self) -> &tempfile::TempDir {
        &self.0
    }
}

impl ProblemDir for WorkSpace {
    fn path(&self) -> Box<path::Path> {
        self.problem_dir()
    }
}

impl SourceDir for WorkSpace {
    fn path(&self) -> Box<path::Path> {
        self.source_dir()
    }
}
