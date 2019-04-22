use std::fmt;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(tag = "type")]
pub enum Problem {
    Normal {
        time_limit: u64,   // NS
        memory_limit: u64, // Bytes
        test_cases: Vec<TestCase>,
    },
    Special {
        time_limit: u64,   // NS
        memory_limit: u64, // Bytes
        test_cases: Vec<TestCase>,
        spj: Source,
    },
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct JudgeTask {
    pub id: String,
    pub source: Source,
    pub problem: Problem,
}

impl Problem {
    pub fn len(&self) -> usize {
        use Problem::*;
        match self {
            Normal { test_cases, .. } => test_cases.len(),
            Special { test_cases, .. } => test_cases.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
pub struct JudgeReport {
    pub id: String,
    pub index: usize,
    pub status: String,
    pub time: u64,   // NS
    pub memory: u64, // Bytes
}

impl JudgeReport {
    pub fn new(id: &str, index: usize, status: JudgeResult, time: u64, memory: u64) -> JudgeReport {
        JudgeReport {
            id: id.to_string(),
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
