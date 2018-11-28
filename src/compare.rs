use std::fs;
use std::io::prelude::*;
use std::path;
use std::process;

pub struct Comparer {}

fn diff(output: &str, answer: &str) -> bool {
    !output
        .trim_end_matches('\n')
        .split_terminator('\n')
        .zip(answer.trim_end_matches('\n').split_terminator('\n'))
        .all(|(output, answer)| output.trim_end_matches(' ') == answer.trim_end_matches(' '))
}

impl Comparer {
    pub fn check(
        input_file: &path::Path,
        output_file: &path::Path,
        answer_file: &path::Path,
        spj: &Option<&path::Path>,
    ) -> bool {
        match spj {
            Some(spj) => process::Command::new(spj)
                .arg(input_file)
                .arg(output_file)
                .arg(answer_file)
                .status()
                .expect("Failed to run special judge")
                .success(),
            None => {
                let mut output = String::new();
                fs::File::open(output_file)
                    .expect("Failed to open output file")
                    .read_to_string(&mut output)
                    .expect("Failed to read output content to string");

                let mut answer = String::new();
                fs::File::open(answer_file)
                    .expect("Failed to open answer file")
                    .read_to_string(&mut answer)
                    .expect("Failed to read answer content to string");

                diff(&output, &answer)
            }
        }
    }
}
