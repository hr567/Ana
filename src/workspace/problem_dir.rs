use std::fs;
use std::path::Path;

use super::SourceDir;
use crate::mtp;

const NS_PER_SEC: f64 = 1_000_000_000 as f64;
const BYTES_PER_MB: f64 = (1024 * 1024) as f64;

pub trait ProblemDir {
    fn path(&self) -> Box<Path>;

    fn time_limit_file(&self) -> Box<Path> {
        self.path().join("time_limit").into_boxed_path()
    }

    fn memory_limit_file(&self) -> Box<Path> {
        self.path().join("memory_limit").into_boxed_path()
    }

    fn spj_path(&self) -> Box<Path> {
        self.path().join("spj").into_boxed_path()
    }

    fn init_problem_dir(&self, problem: mtp::Problem) {
        match problem {
            mtp::Problem::Normal {
                time_limit,
                memory_limit,
                test_cases,
            } => {
                self.generate_limit(time_limit, memory_limit);
                self.generate_test_cases(test_cases);
            }
            mtp::Problem::Special {
                time_limit,
                memory_limit,
                test_cases,
                spj,
            } => {
                self.generate_limit(time_limit, memory_limit);
                self.generate_test_cases(test_cases);
                self.generate_special_judge(spj);
            }
        }
    }

    fn generate_limit(&self, time_limit: f64, memory_limit: f64) {
        fs::write(
            self.time_limit_file(),
            format!("{}", (time_limit * NS_PER_SEC) as u64),
        )
        .unwrap();
        fs::write(
            self.memory_limit_file(),
            format!("{}", (memory_limit * BYTES_PER_MB) as u64),
        )
        .unwrap();
    }

    fn generate_test_cases(&self, test_cases: Vec<mtp::TestCase>) {
        for (index, test_case) in test_cases.iter().enumerate() {
            let test_case_path = self.path().join(index.to_string());
            fs::create_dir(&test_case_path).unwrap();
            fs::write(&test_case_path.join("input"), &test_case.input).unwrap();
            fs::write(&test_case_path.join("answer"), &test_case.answer).unwrap();
        }
    }

    fn generate_special_judge(&self, spj: mtp::Source) {
        let spj_path = self.spj_path();
        fs::create_dir(&spj_path).unwrap();
        spj_path.init_source_dir(spj);
    }

    fn test_cases(&self) -> Vec<Box<Path>> {
        let mut res = Vec::new();
        for index in 0.. {
            let test_case_path = self.path().join(index.to_string());
            if test_case_path.exists() {
                res.push(test_case_path.into_boxed_path());
            } else {
                break;
            }
        }
        res
    }

    fn get_time_limit(&self) -> u64 {
        let time_limit = fs::read(self.time_limit_file()).unwrap();
        String::from_utf8(time_limit)
            .unwrap()
            .trim()
            .parse()
            .unwrap()
    }

    fn get_memory_limit(&self) -> u64 {
        let memory_limit = fs::read(self.memory_limit_file()).unwrap();
        String::from_utf8(memory_limit)
            .unwrap()
            .trim()
            .parse()
            .unwrap()
    }
}

impl ProblemDir for Path {
    fn path(&self) -> Box<Path> {
        self.to_path_buf().into_boxed_path()
    }
}
