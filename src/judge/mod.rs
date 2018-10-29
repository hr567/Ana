mod compile;

use std::env;

use self::compile::{compile, Languages};
use super::problem::Problem;

pub enum JudgeResult {
    CE,
    AC,
    WA,
    TLE,
    MLE,
    RE,
}

pub fn judge(
    problem: &Problem,
    source_code: &str,
    language: &Languages,
) -> (JudgeResult, Vec<JudgeResult>) {
    let mut executable_file = env::temp_dir();
    executable_file.push(&problem.executable_filename);
    executable_file.set_extension("exe");

    match compile(language, source_code, executable_file.as_path()) {
        Ok(_) => {}
        Err(_) => {}
    }

    (JudgeResult::AC, vec![])
}
