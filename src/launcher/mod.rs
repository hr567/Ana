mod back_ends;

use std::process::Output;

#[derive(PartialEq, Debug)]
pub enum LaunchResult {
    Pass(Output),
    TLE,
    MLE,
    RE,
}

trait Launcher {
    fn run(&self) -> LaunchResult;
}

pub struct Limit {
    time: f64,
    memory: f64,
}

impl Limit {
    pub fn new(time: f64, memory: f64) -> Self {
        Limit { time, memory }
    }
}
