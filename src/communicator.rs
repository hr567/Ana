/// Simple traits for communicate between server and client
use std::sync::{Arc, Mutex};

use crate::mtp;

pub const EOF: &str = "EOF";

#[derive(PartialEq, Debug)]
pub enum Error {
    Network,
    Data,
    EOF,
}

/// The `Receiver` trait allows for receiving judge task
pub trait Receiver {
    /// Block the current thread and then
    /// receive a judge task.
    fn receive(&self) -> Result<mtp::JudgeTask, Error>;
}

/// The `Sender` trait allows for sending judge report
pub trait Sender {
    /// Send a JudgeReport
    fn send(&self, report: mtp::JudgeReport) -> Result<(), Error>;
}

/// A wrapper for task receiver which implemented Stream trait.
pub struct TaskReceiver<T: Receiver>(T);

impl<T: Receiver> Receiver for TaskReceiver<T> {
    fn receive(&self) -> Result<mtp::JudgeTask, Error> {
        self.0.receive()
    }
}

impl<T: Receiver> From<T> for TaskReceiver<T> {
    fn from(inner: T) -> TaskReceiver<T> {
        TaskReceiver(inner)
    }
}

/// A wrapper for report receiver.
/// It is able to be cloned and be sent between threads.
pub struct ReportSender<T: Sender>(Arc<Mutex<T>>);

impl<T: Sender> Sender for ReportSender<T> {
    fn send(&self, report: mtp::JudgeReport) -> Result<(), Error> {
        self.0.lock().unwrap().send(report)
    }
}

impl<T: Sender> Clone for ReportSender<T> {
    fn clone(&self) -> ReportSender<T> {
        ReportSender(self.0.clone())
    }
}

impl<T: Sender> From<T> for ReportSender<T> {
    fn from(inner: T) -> ReportSender<T> {
        ReportSender(Arc::new(Mutex::new(inner)))
    }
}
