pub struct Limit {
    pub time: f64,   // Sec
    pub memory: f64, // MB
}

impl Limit {
    pub fn new(time: f64, memory: f64) -> Self {
        Limit { time, memory }
    }
}
