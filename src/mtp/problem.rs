use serde_derive::{Deserialize, Serialize};

use super::Source;

pub enum ProblemType {
    Normal,
    Special,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Problem {
    pub problem_type: String,
    pub time_limit: f64,
    pub memory_limit: f64,
    pub checker: Source,
    pub test_cases: Vec<TestCase>,
}

impl Problem {
    pub fn get_type(&self) -> ProblemType {
        match self.problem_type.as_str() {
            "normal" => ProblemType::Normal,
            "spj" => ProblemType::Special,
            _ => unimplemented!("Not support problem type {}", self.problem_type),
        }
    }

    pub fn len(&self) -> usize {
        self.test_cases.len()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TestCase {
    pub input: String,
    pub answer: String,
}
