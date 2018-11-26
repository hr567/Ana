use super::judge::JudgeReport;

mod judge;
mod problem;
mod report;
mod source;

pub use self::{
    judge::JudgeInfo,
    problem::{Problem, TestCase},
    report::ReportInfo,
    source::Source,
};
