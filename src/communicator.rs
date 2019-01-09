use super::mtp::{JudgeInfo, ReportInfo};

pub trait JudgeReceiver {
    fn receive_judge_information(&self) -> Option<JudgeInfo>;
}

pub trait ReportSender {
    fn send_report_information(&self, report: ReportInfo);
}
