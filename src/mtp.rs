use std::fmt;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct JudgeInfo {
    pub id: String,
    pub source: Source,
    pub problem: Problem,
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Source {
    pub language: String,
    pub code: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReportInfo {
    pub id: String,
    pub index: usize,
    pub status: String,
    pub time: f64,
    pub memory: f64,
}

impl ReportInfo {
    pub fn new(id: &str, index: usize, status: JudgeResult, time: f64, memory: f64) -> ReportInfo {
        ReportInfo {
            id: String::from(id),
            index,
            status: status.to_string(),
            time,
            memory,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum JudgeResult {
    CE,
    AC,
    WA,
    TLE,
    MLE,
    OLE,
    RE,
}

impl fmt::Display for JudgeResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use JudgeResult::*;
        write!(
            f,
            "{}",
            match self {
                AC => "AC",
                CE => "CE",
                MLE => "MLE",
                OLE => "OLE",
                RE => "RE",
                TLE => "TLE",
                WA => "WA",
            }
        )
    }
}
