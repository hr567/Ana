use std::fs;
use std::io;
use std::io::prelude::*;
use std::path;
use std::process;

pub struct Comparer;

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
    ) -> io::Result<bool> {
        match spj {
            Some(spj) => Ok(process::Command::new(spj)
                .arg(input_file)
                .arg(answer_file)
                .arg(output_file)
                .status()?
                .success()),
            None => {
                let mut output = String::new();
                fs::File::open(output_file)?.read_to_string(&mut output)?;
                let mut answer = String::new();
                fs::File::open(answer_file)?.read_to_string(&mut answer)?;
                Ok(!diff(&output, &answer))
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
    fn test_check_without_spj() -> io::Result<()> {
        let file0 = env::temp_dir().join("test_check_without_spj.0");
        let file1 = env::temp_dir().join("test_check_without_spj.1");
        let file2 = env::temp_dir().join("test_check_without_spj.2");
        fs::write(&file0, "hello world")?;
        fs::write(&file1, "hello world")?;
        fs::write(&file2, "helloworld")?;
        assert!(Comparer::check(&env::temp_dir(), &file0, &file1, &None)?);
        assert!(!Comparer::check(&env::temp_dir(), &file0, &file2, &None)?);
        fs::remove_file(&file0)?;
        fs::remove_file(&file1)?;
        fs::remove_file(&file2)?;
        Ok(())
    }

    // TODO: Add Spj test
    // #[test]
    // fn test_check_with_spj() -> io::Result<()> {
    //     unimplemented!()
    // }
}
