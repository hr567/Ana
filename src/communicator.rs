use super::mtp;

pub trait JudgeReceiver {
    fn receive_judge_information(&self) -> Option<mtp::JudgeInfo>;
}

pub trait ReportSender {
    fn send_report_information(&self, report: mtp::ReportInfo);
}

impl JudgeReceiver for zmq::Socket {
    fn receive_judge_information(&self) -> Option<mtp::JudgeInfo> {
        let judge_info = self
            .recv_string(0)
            .expect("Failed to receive the judge information")
            .expect("Received message is not a string");
        if let Ok(judge_info) = serde_json::from_str(&judge_info) {
            Some(judge_info)
        } else {
            None
        }
    }
}

impl ReportSender for zmq::Socket {
    fn send_report_information(&self, report: mtp::ReportInfo) {
        self.send(&report.to_json(), 0)
            .expect("Failed to send the report information");
    }
}
