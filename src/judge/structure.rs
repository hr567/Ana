use std::fmt;

pub struct JudgeReport {
    pub status: JudgeResult,
    pub time: f64,
    pub memory: u64,
}

impl JudgeReport {
    pub fn new(status: JudgeResult, time: f64, memory: u64) -> JudgeReport {
        JudgeReport {
            status,
            time,
            memory,
        }
    }
}

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
        write!(
            f,
            "{}",
            match self {
                JudgeResult::AC => "AC",
                JudgeResult::CE => "CE",
                JudgeResult::MLE => "MLE",
                JudgeResult::OLE => "OLE",
                JudgeResult::RE => "RE",
                JudgeResult::TLE => "TLE",
                JudgeResult::WA => "WA",
            }
        )
    }
}
