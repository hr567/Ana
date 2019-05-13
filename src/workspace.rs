/// Workspace on tmpfs
///
/// TODO: Use fuse rewrite in the future for
/// better performance and less memory usage
use std::fs;
use std::path;

use libmount;
use tempfile;

use crate::mtp;

const BYTES_PER_MB: usize = 1024 * 1024;

pub struct Workspace {
    inner: tempfile::TempDir,
}

impl Workspace {
    pub fn new() -> Workspace {
        Workspace::default()
    }
}

impl Default for Workspace {
    fn default() -> Workspace {
        let workspace = Workspace {
            inner: tempfile::tempdir().expect("Failed to create a temp dir"),
        };

        libmount::Tmpfs::new(&workspace)
            .mode(0o700)
            .mount()
            .expect("Failed to mount tmpfs on workspace");

        fs::create_dir(&workspace.runtime_dir()).expect("Failed to create runtime directory");
        libmount::Tmpfs::new(&workspace.runtime_dir())
            .size_bytes(32 * BYTES_PER_MB)
            .mode(0o700)
            .mount()
            .expect("Failed to mount tmpfs on runtime directory");

        workspace
    }
}

impl AsRef<path::Path> for Workspace {
    fn as_ref(&self) -> &path::Path {
        self.inner.path()
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        nix::mount::umount(self.runtime_dir().as_ref()).unwrap();
        nix::mount::umount(self.inner.path()).unwrap();
    }
}

pub trait WorkDir {
    fn source_file(&self) -> Box<path::Path>;
    fn problem_dir(&self) -> Box<path::Path>;
    fn runtime_dir(&self) -> Box<path::Path>;

    fn prepare_judge_task(&self, judge_task: &mtp::JudgeTask);
}

pub trait RuntimeDir {
    fn executable_file(&self) -> Box<path::Path>;
}

pub trait ProblemDir {
    fn test_case_dirs(&self) -> Vec<Box<path::Path>>;

    fn prepare_problem(&self, problem: &mtp::Problem);
}

pub trait SpecialJudgeProblemDir: ProblemDir {
    fn spj_file(&self) -> Box<path::Path>;
    fn spj_source(&self) -> Box<path::Path>;

    fn prepare_special_judge_problem(&self, problem: &mtp::Problem);
}

pub trait TestCaseDir {
    fn input_file(&self) -> Box<path::Path>;
    fn output_file(&self) -> Box<path::Path>;
    fn answer_file(&self) -> Box<path::Path>;

    fn prepare_test_case(&self, test_case: &mtp::TestCase);
}

impl WorkDir for Workspace {
    fn source_file(&self) -> Box<path::Path> {
        self.inner.path().join("source").into_boxed_path()
    }

    fn problem_dir(&self) -> Box<path::Path> {
        self.inner.path().join("problem").into_boxed_path()
    }

    fn runtime_dir(&self) -> Box<path::Path> {
        self.inner.path().join("runtime").into_boxed_path()
    }

    fn prepare_judge_task(&self, judge_task: &mtp::JudgeTask) {
        fs::create_dir(self.problem_dir()).unwrap();
        fs::write(self.source_file(), &judge_task.source.code).unwrap();
        self.problem_dir().prepare_problem(&judge_task.problem);
    }
}

impl ProblemDir for path::Path {
    fn test_case_dirs(&self) -> Vec<Box<path::Path>> {
        let mut res = Vec::new();
        for i in 0.. {
            let test_case_dir = self.join(i.to_string());
            if test_case_dir.exists() {
                res.push(test_case_dir.into_boxed_path());
            } else {
                break;
            }
        }
        res
    }

    fn prepare_problem(&self, problem: &mtp::Problem) {
        match problem {
            mtp::Problem::Normal { test_cases, .. } => {
                for (i, test_case) in test_cases.iter().enumerate() {
                    let test_case_dir = self.join(i.to_string());
                    fs::create_dir(&test_case_dir).unwrap();
                    test_case_dir.prepare_test_case(&test_case)
                }
            }
            mtp::Problem::Special { test_cases, .. } => {
                self.prepare_special_judge_problem(problem);
                for (i, test_case) in test_cases.iter().enumerate() {
                    let test_case_dir = self.join(i.to_string());
                    fs::create_dir(&test_case_dir).unwrap();
                    test_case_dir.prepare_test_case(&test_case)
                }
            }
        }
    }
}

impl SpecialJudgeProblemDir for path::Path {
    fn spj_file(&self) -> Box<path::Path> {
        self.join("spj").into_boxed_path()
    }

    fn spj_source(&self) -> Box<path::Path> {
        self.join("spj").into_boxed_path()
    }

    fn prepare_special_judge_problem(&self, problem: &mtp::Problem) {
        if let mtp::Problem::Special { spj, .. } = problem {
            fs::write(self.spj_source(), &spj.code).unwrap();
        }
    }
}

impl RuntimeDir for path::Path {
    fn executable_file(&self) -> Box<path::Path> {
        self.join("main").into_boxed_path()
    }
}

impl TestCaseDir for path::Path {
    fn input_file(&self) -> Box<path::Path> {
        self.join("input").into_boxed_path()
    }

    fn output_file(&self) -> Box<path::Path> {
        self.join("output").into_boxed_path()
    }

    fn answer_file(&self) -> Box<path::Path> {
        self.join("answer").into_boxed_path()
    }

    fn prepare_test_case(&self, test_case: &mtp::TestCase) {
        fs::write(self.input_file(), &test_case.input).unwrap();
        fs::write(self.answer_file(), &test_case.answer).unwrap();
    }
}
