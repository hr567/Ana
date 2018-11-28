use std::env;

use serde_derive::{Deserialize, Serialize};

use super::JudgeReport;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ReportInfo {
    pub id: String,
    pub case_index: usize,
    pub status: String,
    pub time: f64,
    pub memory: u64,
}

impl ReportInfo {
    pub fn new(case_index: usize, report: &JudgeReport) -> ReportInfo {
        ReportInfo {
            id: env::var("ANA_JUDGE_ID").unwrap(),
            case_index,
            status: report.status.to_string(),
            time: report.time,
            memory: report.memory,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
