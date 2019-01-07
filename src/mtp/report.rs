use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ReportInfo {
    pub id: String,
    pub index: usize,
    pub status: String,
    pub time: f64,
    pub memory: f64,
}

impl ReportInfo {
    pub fn new(id: &str, index: usize, status: &str, time: f64, memory: f64) -> ReportInfo {
        ReportInfo {
            id: String::from(id),
            index,
            status: String::from(status),
            time,
            memory,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
