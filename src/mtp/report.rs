use std::fmt;

use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ReportInfo {
    pub id: String,
    pub index: usize,
    pub status: &'static str,
    pub time: f64,
    pub memory: f64,
}

impl ReportInfo {
    pub fn new(id: &str, index: usize, status: JudgeResult, time: f64, memory: f64) -> ReportInfo {
        ReportInfo {
            id: String::from(id),
            index,
            status: &status.as_str(),
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

impl JudgeResult {
    fn as_str(&self) -> &'static str {
        match self {
            JudgeResult::AC => "AC",
            JudgeResult::CE => "CE",
            JudgeResult::MLE => "MLE",
            JudgeResult::OLE => "OLE",
            JudgeResult::RE => "RE",
            JudgeResult::TLE => "TLE",
            JudgeResult::WA => "WA",
        }
    }
}

impl fmt::Display for JudgeResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
