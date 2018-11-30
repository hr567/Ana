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
                .arg(answer_file)
                .arg(output_file)
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

                !diff(&output, &answer)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn test_diff_complete_eq() {
        assert!(!diff("hello world", "hello world"));
    }

    #[test]
    fn test_diff_with_empty_line_at_eof() {
        assert!(!diff("hello world\n", "hello world"));
        assert!(!diff("hello world", "hello world\n"));
    }

    #[test]
    fn test_diff_with_space_at_eol() {
        assert!(!diff("hello world ", "hello world"));
        assert!(!diff("hello world", "hello world "));
    }

    #[test]
    fn test_diff_with_both_empty_line_at_eof_and_space_at_eol() {
        assert!(!diff("hello world \n", "hello world"));
        assert!(!diff("hello world", "hello world \n"));
        assert!(!diff("hello world\n", "hello world "));
        assert!(!diff("hello world ", "hello world\n"));
        assert!(!diff("hello world \n", "hello world\n"));
        assert!(!diff("hello world\n", "hello world \n"));
        assert!(!diff("hello world\n ", "hello world\n"));
        assert!(!diff("hello world\n", "hello world\n "));
    }

    #[test]
    fn test_check_without_spj() {
        env::set_var("ANA_WORK_DIR", env::temp_dir());
        let work_dir = path::PathBuf::from(env::var("ANA_WORK_DIR").unwrap());
        let mut input_file = path::PathBuf::new();
        input_file.clone_from(&work_dir);
        input_file.push("test_check_without_spj");
        input_file.set_extension("in");

        let mut output_file = path::PathBuf::new();
        output_file.clone_from(&work_dir);
        output_file.push("test_check_without_spj");
        output_file.set_extension("out");
        fs::File::create(&output_file)
            .unwrap()
            .write_all(b"hello world")
            .unwrap();

        let mut answer_file = path::PathBuf::new();
        answer_file.clone_from(&work_dir);
        answer_file.push("test_check_without_spj");
        answer_file.set_extension("ans");
        fs::File::create(&answer_file)
            .unwrap()
            .write_all(b"hello world")
            .unwrap();

        assert!(Comparer::check(
            &input_file,
            &output_file,
            &answer_file,
            &None,
        ));
    }
}
