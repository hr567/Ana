use std::sync;

use crate::mtp;
use tokio::prelude::*;

#[cfg(feature = "zmq_mod")]
mod zmq;

#[derive(PartialEq, Debug)]
pub enum Error {
    Network,
    Data,
}

/// The `Receiver` trait allows for receiving judge task
pub trait Receiver {
    /// Receive a JudgeTask (may block current thread)
    fn receive(&self) -> Result<mtp::JudgeTask, Error>;
}

/// The `Sender` trait allows for sending judge report
pub trait Sender {
    /// Send a JudgeReport (may block current thread)
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
    type Error = ();
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match tokio_threadpool::blocking(|| self.receive()) {
            Ok(Async::Ready(Ok(res))) => Ok(Async::Ready(Some(res))),
            Ok(Async::Ready(Err(Error::Data))) => Ok(Async::Ready(None)),
            Ok(Async::Ready(Err(Error::Network))) => Err(()),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(_) => Err(()),
        }
    }
}

pub struct ReportSender<T: Sender>(sync::Arc<sync::Mutex<T>>);

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
        ReportSender(sync::Arc::new(sync::Mutex::new(inner)))
    }
}
