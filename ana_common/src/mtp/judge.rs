use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct JudgeInfo {
    pub id: String,
    pub source: Source,
    pub problem: Problem,
}

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

    pub fn is_empty(&self) -> bool {
        self.test_cases.is_empty()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TestCase {
    pub input: String,
    pub answer: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Source {
    pub language: String,
    pub code: String,
}
