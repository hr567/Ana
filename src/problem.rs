use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TestCase {
    input: String,
    answer: String,
}

#[derive(Serialize, Deserialize)]
pub struct Problem {
    id: u64,
    name: String,
    input_filename: String,
    output_filename: String,
    time_limit: f64,
    memory_limit: f64,
    stdio: bool,
    optimize: bool,
    test_cases: Vec<TestCase>,
}

impl Problem {
    pub fn new(
        id: u64,
        name: String,
        input_filename: String,
        output_filename: String,
        time_limit: f64,
        memory_limit: f64,
        stdio: bool,
        optimize: bool,
        test_cases: Vec<TestCase>,
    ) -> Self {
        Problem {
            id,
            name,
            input_filename,
            output_filename,
            time_limit,
            memory_limit,
            stdio,
            optimize,
            test_cases,
        }
    }

    pub fn from_json(json: &str) -> Self {
        serde_json::from_str(json).expect("Fail to deserialize json")
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Fail to serialize the problem")
    }
}
