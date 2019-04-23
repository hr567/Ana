use std::sync::{Arc, Mutex};

use crate::mtp;
use tokio::prelude::*;

pub const EOF: &str = "EOF";

#[derive(PartialEq, Debug)]
pub enum Error {
    Network,
    Data,
    EOF,
}

/// The `Receiver` trait allows for receiving judge task
pub trait Receiver {
    /// Receive a JudgeTask
    /// This method should block current thread until
    /// it receive a judge task and then return it.
    fn receive(&self) -> Result<mtp::JudgeTask, Error>;
}

/// The `Sender` trait allows for sending judge report
pub trait Sender {
    /// Send a JudgeReport
    fn send(&self, report: mtp::JudgeReport) -> Result<(), Error>;
}

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

impl<T: Receiver> Stream for TaskReceiver<T> {
    type Item = mtp::JudgeTask;
    type Error = Error;
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match tokio_threadpool::blocking(|| self.receive())
            .expect("Panic when trying to receive a task")
        {
            Async::Ready(Ok(res)) => Ok(Async::Ready(Some(res))),
            Async::Ready(Err(Error::Data)) => Err(Error::Data),
            Async::Ready(Err(Error::Network)) => Err(Error::Network),
            Async::Ready(Err(Error::EOF)) => Ok(Async::Ready(None)),
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}

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
