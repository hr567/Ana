use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TestCase {
    input: String,
    answer: String,
}

#[derive(Serialize, Deserialize)]
pub struct Problem {
    pub id: u64,
    pub name: String,
    pub source_filename: String,
    pub executable_filename: String,
    pub input_filename: String,
    pub output_filename: String,
    pub time_limit: f64,
    pub memory_limit: f64,
    pub stdio: bool,
    pub optimize: bool,
    pub test_cases: Vec<TestCase>,
}

impl Problem {
    pub fn new(
        id: u64,
        name: String,
        source_filename: String,
        executable_filename: String,
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
            source_filename,
            executable_filename,
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

    pub fn test_case_count(&self) -> usize {
        self.test_cases.len()
    }
}
