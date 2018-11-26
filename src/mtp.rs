// TODO: Add support for other kinds of problem
use serde_derive::{Deserialize, Serialize};

use super::judge::JudgeReport;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ReportInfo {
    pub id: String,
    pub case_index: usize,
    pub status: &'static str,
    pub time: f64,
    pub memory: u64,
}

impl ReportInfo {
    pub fn new(id: &str, case_index: usize, report: &JudgeReport) -> ReportInfo {
        ReportInfo {
            id: id.to_string(),
            case_index,
            status: report.status.to_str(),
            time: report.time,
            memory: report.memory,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct JudgeInfo {
    pub id: String,
    pub source: Source,
    pub problem: Problem,
}

impl JudgeInfo {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Problem {
    pub problem_type: String,
    pub time_limit: f64,
    pub memory_limit: f64,
    pub checker: Source,
    pub test_cases: Vec<TestCase>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Source {
    pub language: String,
    pub code: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TestCase {
    pub input: String,
    pub answer: String,
}
