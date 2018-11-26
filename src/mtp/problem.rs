use serde_derive::{Deserialize, Serialize};

use super::Source;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Problem {
    pub problem_type: String,
    pub time_limit: f64,
    pub memory_limit: f64,
    pub checker: Source,
    pub test_cases: Vec<TestCase>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TestCase {
    pub input: String,
    pub answer: String,
}
