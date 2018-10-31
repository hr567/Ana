mod back_ends;

use std::process::Output;

trait Launcher {
    fn run(&self) -> Output;
}

struct Limit {
    time: f64,
    memory: f64,
}

impl Limit {
    fn new(time: f64, memory: f64) -> Self {
        Limit { time, memory }
    }
}
