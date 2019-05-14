/// Utils to compare two files line by line
use std::fs;
use std::io::{self, BufRead};
use std::path;
use std::process;

pub struct Checker<'a> {
    ignore_white_space_at_line_end: bool,
    ignore_empty_line_at_file_end: bool,
    extern_program: Option<&'a path::Path>,
}

impl<'a> Default for Checker<'a> {
    fn default() -> Checker<'a> {
        Checker {
            ignore_white_space_at_line_end: true,
            ignore_empty_line_at_file_end: true,
            extern_program: None,
        }
    }
}

/// A configurable content comparer
impl<'a> Checker<'a> {
    pub fn new() -> Checker<'a> {
        Checker::default()
    }

    pub fn ignore_white_space_at_line_end(mut self, flag: bool) -> Checker<'a> {
        self.ignore_white_space_at_line_end = flag;
        self
    }

    pub fn ignore_empty_line_at_file_end(mut self, flag: bool) -> Checker<'a> {
        self.ignore_empty_line_at_file_end = flag;
        self
    }

    /// Use an extern program to check the output
    pub fn extern_program(mut self, program: &'a path::Path) -> Checker<'a> {
        self.extern_program = Some(program);
        self
    }

    /// Compare output file and answer file line by line
    ///
    /// Return `true` if there is no difference between two files
    pub fn compare_files(
        &self,
        output_file: &path::Path,
        answer_file: &path::Path,
    ) -> io::Result<bool> {
        let mut output_buf = {
            let file = fs::File::open(output_file)?;
            io::BufReader::new(file)
        };

        let mut answer_buf = {
            let file = fs::File::open(answer_file)?;
            io::BufReader::new(file)
        };

        Ok(!self.buf_diff(&mut output_buf, &mut answer_buf)?)
    }

    /// Compare two strings
    ///
    /// Return `true` if there is no difference between two strings
    pub fn compare_strings(&self, output: &str, answer: &str) -> io::Result<bool> {
        self.compare_bytes(output.as_bytes(), answer.as_bytes())
    }

    /// Compare two bytes arrays
    ///
    /// Return `true` if there is no difference between two arrays
    pub fn compare_bytes(&self, output: &[u8], answer: &[u8]) -> io::Result<bool> {
        let mut output_buf = io::BufReader::new(output);
        let mut answer_buf = io::BufReader::new(answer);
        Ok(!self.buf_diff(&mut output_buf, &mut answer_buf)?)
    }

    /// Check output by provided extern program
    ///
    /// The result is completely depended on the extern program.
    /// So the ignore_* options of the checker will be ignored.
    ///
    /// Return `true` if extern program exit successfully
    pub fn check_use_extern_program(
        &self,
        input_file: &path::Path,
        output_file: &path::Path,
        answer_file: &path::Path,
    ) -> io::Result<bool> {
        if let Some(spj) = self.extern_program {
            let check_result = process::Command::new(spj)
                .arg(input_file)
                .arg(answer_file)
                .arg(output_file)
                .status()?
                .success();
            Ok(check_result)
        } else {
            panic!("Checker's extern program has not been set")
        }
    }

    /// Check if two buffers is equal line by line
    ///
    /// Handle the blank line at the end of file
    /// and white space at the end of line
    /// by the checker's options.
    ///
    /// Return `false` if there is no difference between two buffers
    fn buf_diff<T: BufRead, U: BufRead>(
        &self,
        output_buf: &mut T,
        answer_buf: &mut U,
    ) -> io::Result<bool> {
        loop {
            let get_line = |buf_reader: &mut BufRead| -> io::Result<Option<Vec<u8>>> {
                let mut buf = Vec::new();
                buf_reader.read_until(b'\n', &mut buf)?;

                if buf.is_empty() {
                    return Ok(None);
                }

                if buf.ends_with(&[b'\n']) {
                    buf.pop();
                }

                if self.ignore_white_space_at_line_end {
                    while buf.ends_with(&[b' ']) {
                        buf.pop();
                    }
                }

                Ok(Some(buf))
            };

            match (get_line(output_buf)?, get_line(answer_buf)?) {
                (Some(output), Some(answer)) => {
                    if output != answer {
                        return Ok(true);
                    }
                    continue;
                }
                (Some(output), None) => {
                    if !output.is_empty() || !self.ignore_empty_line_at_file_end {
                        return Ok(true);
                    }
                    continue;
                }
                (None, Some(answer)) => {
                    if !answer.is_empty() || !self.ignore_empty_line_at_file_end {
                        return Ok(true);
                    }
                    continue;
                }
                (None, None) => {
                    break;
                }
            }
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile;

    fn diff(output: &[u8], answer: &[u8]) -> bool {
        !Checker::default().compare_bytes(&output, &answer).unwrap()
    }

    #[test]
    fn test_diff_complete_eq() {
        assert!(!diff(b"hello world", b"hello world"));
    }

    #[test]
    fn test_diff_with_empty_line_at_eof() {
        assert!(!diff(b"hello world\n", b"hello world"));
        assert!(!diff(b"hello world\n\n", b"hello world"));
        assert!(!diff(b"hello world\n", b"hello world\n"));
        assert!(!diff(b"hello world\n\n", b"hello world\n"));
        assert!(!diff(b"hello world\n\n", b"hello world\n\n"));
        assert!(!diff(b"hello world", b"hello world\n"));
        assert!(!diff(b"hello world", b"hello world\n\n"));
    }

    #[test]
    fn test_diff_with_space_at_eol() {
        assert!(!diff(b"hello world ", b"hello world"));
        assert!(!diff(b"hello world  ", b"hello world"));
        assert!(!diff(b"hello world", b"hello world  "));
    }

    #[test]
    fn test_diff_with_both_empty_line_at_eof_and_space_at_eol() {
        assert!(!diff(b"hello world \n", b"hello world"));
        assert!(!diff(b"hello world", b"hello world \n"));
        assert!(!diff(b"hello world\n", b"hello world "));
        assert!(!diff(b"hello world ", b"hello world\n"));
        assert!(!diff(b"hello world \n", b"hello world\n"));
        assert!(!diff(b"hello world\n", b"hello world \n"));
        assert!(!diff(b"hello world\n ", b"hello world\n"));
        assert!(!diff(b"hello world\n", b"hello world\n "));
        assert!(!diff(b"hello world\n \n", b"hello world\n"));
        assert!(!diff(b"hello world\n", b"hello world\n \n"));
    }

    #[test]
    fn test_check() -> io::Result<()> {
        let work_dir = tempfile::tempdir()?;
        let file0 = work_dir.path().join("test_check_without_spj.0");
        let file1 = work_dir.path().join("test_check_without_spj.1");
        let file2 = work_dir.path().join("test_check_without_spj.2");
        fs::write(&file0, "hello world")?;
        fs::write(&file1, "hello world")?;
        fs::write(&file2, "hello_world")?;

        let checker = Checker::default();
        assert!(checker.compare_files(&file0, &file1)?);
        assert!(!checker.compare_files(&file0, &file2)?);
        Ok(())
    }
}
