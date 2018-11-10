// TODO: Add support for other kinds of problem

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TestCase {
    pub input: String,
    pub answer: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
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

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn test_case_count(&self) -> usize {
        self.test_cases.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let json_problem: &str = r#"{"id":1000,"name":"Example A+B Problem","source_filename":"main","executable_filename":"main","input_filename":"example.in","output_filename":"example.out","time_limit":1.0,"memory_limit":128.0,"stdio":true,"optimize":true,"test_cases":[{"input":"1 1","answer":"2"},{"input":"13 5\n14 7\n23 45","answer":"18\n21\n68"}]}"#;
        let problem = Problem::new(
            1000,
            String::from("Example A+B Problem"),
            String::from("main"),
            String::from("main"),
            String::from("example.in"),
            String::from("example.out"),
            1.0,
            128.0,
            true,
            true,
            vec![
                TestCase {
                    input: String::from("1 1"),
                    answer: String::from("2"),
                },
                TestCase {
                    input: String::from("13 5\n14 7\n23 45"),
                    answer: String::from("18\n21\n68"),
                },
            ],
        );
        assert_eq!(problem.to_json().unwrap().as_str(), json_problem);
    }

    #[test]
    fn test_deserialize() {
        let json_problem: &str = r#"{"id":1000,"name":"Example A+B Problem","source_filename":"main","executable_filename":"main","input_filename":"example.in","output_filename":"example.out","time_limit":1.0,"memory_limit":128.0,"stdio":true,"optimize":true,"test_cases":[{"input":"1 1","answer":"2"},{"input":"13 5\n14 7\n23 45","answer":"18\n21\n68"}]}"#;
        let problem = Problem::new(
            1000,
            String::from("Example A+B Problem"),
            String::from("main"),
            String::from("main"),
            String::from("example.in"),
            String::from("example.out"),
            1.0,
            128.0,
            true,
            true,
            vec![
                TestCase {
                    input: String::from("1 1"),
                    answer: String::from("2"),
                },
                TestCase {
                    input: String::from("13 5\n14 7\n23 45"),
                    answer: String::from("18\n21\n68"),
                },
            ],
        );
        assert_eq!(Problem::from_json(json_problem).unwrap(), problem);
    }
}
