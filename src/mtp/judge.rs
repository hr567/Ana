use serde_derive::{Deserialize, Serialize};

use super::{Problem, Source};

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
