use serde_derive::{Deserialize, Serialize};

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
    pub fn new(id: &str, case_index: usize, status: &str, time: f64, memory: f64) -> ReportInfo {
        ReportInfo {
            id: String::from(id),
            case_index,
            status: String::from(status),
            time: time / US_PER_SEC,
            memory: memory / BYTES_PER_MB,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
