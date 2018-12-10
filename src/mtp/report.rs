use std::env;

use serde_derive::{Deserialize, Serialize};

use super::JudgeReport;

const US_PER_SEC: f64 = (1000 * 1000) as f64;
const BYTES_PER_MB: f64 = (1024 * 1024) as f64;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ReportInfo {
    pub id: String,
    pub case_index: usize,
    pub status: String,
    pub time: f64,
    pub memory: f64,
}

impl ReportInfo {
    pub fn new(case_index: usize, report: &JudgeReport) -> ReportInfo {
        ReportInfo {
            id: env::var("ANA_JUDGE_ID").unwrap(),
            case_index,
            status: report.status.to_string(),
            time: report.time as f64 / US_PER_SEC,
            memory: report.memory as f64 / BYTES_PER_MB,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
