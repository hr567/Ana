use serde_derive::{Deserialize, Serialize};

use super::JudgeReport;

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
