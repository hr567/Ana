//! Utils to compare two files/strings line by line.
use std::char;
use std::io;
use std::marker::Unpin;
use std::path::Path;

use tokio::fs::File;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, BufReader};

pub struct Comparer {
    ignore_white_space_at_eol: bool,
    ignore_empty_line_at_eof: bool,
}

impl Comparer {
    pub fn new(ignore_white_space_at_eol: bool, ignore_empty_line_at_eof: bool) -> Comparer {
        Comparer {
            ignore_white_space_at_eol,
            ignore_empty_line_at_eof,
        }
    }

    /// Compare output file and answer file line by line.
    ///
    /// Return `true` if there is no difference between two files.
    pub async fn compare_files(
        &self,
        output_file: impl AsRef<Path>,
        answer_file: impl AsRef<Path>,
    ) -> io::Result<bool> {
        let output_file = File::open(&output_file).await?;
        let answer_file = File::open(&answer_file).await?;
        let mut output_buf = BufReader::new(output_file);
        let mut answer_buf = BufReader::new(answer_file);
        Ok(!self.buf_diff(&mut output_buf, &mut answer_buf).await?)
    }

    /// Compare two bytes.
    ///
    /// Return `true` if there is no difference between them.
    pub async fn compare<S: AsRef<[u8]>>(&self, output: S, answer: S) -> bool {
        let mut output_buf = BufReader::new(output.as_ref());
        let mut answer_buf = BufReader::new(answer.as_ref());
        !self
            .buf_diff(&mut output_buf, &mut answer_buf)
            .await
            .unwrap()
    }
}

impl Comparer {
    /// Check if two buffers is equal line by line.
    ///
    /// Handle the blank line at the end of file
    /// and white space at the end of line
    /// by the checker's options.
    ///
    /// Return `false` if there is no difference between two buffers.
    async fn buf_diff<'a>(
        &self,
        output_buf: &'a mut (dyn AsyncBufRead + Unpin + Send + Sync),
        answer_buf: &'a mut (dyn AsyncBufRead + Unpin + Send + Sync),
    ) -> io::Result<bool> {
        loop {
            let output = {
                let mut buf = Vec::new();
                output_buf.read_until(b'\n', &mut buf).await?;
                if buf.is_empty() {
                    None
                } else {
                    Some(buf)
                }
            };
            let answer = {
                let mut buf = Vec::new();
                answer_buf.read_until(b'\n', &mut buf).await?;
                if buf.is_empty() {
                    None
                } else {
                    Some(buf)
                }
            };

            let mut output = output.as_deref();
            let mut answer = answer.as_deref();
            if self.ignore_white_space_at_eol {
                output = output.map(|s| trim_end(s));
                answer = answer.map(|s| trim_end(s));
            }

            match (output, answer) {
                (Some(output), Some(answer)) => {
                    if output != answer {
                        return Ok(true);
                    }
                    continue;
                }
                (Some(output), None) => {
                    if !output.is_empty() || !self.ignore_empty_line_at_eof {
                        return Ok(true);
                    }
                    continue;
                }
                (None, Some(answer)) => {
                    if !answer.is_empty() || !self.ignore_empty_line_at_eof {
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

fn trim_end(s: &[u8]) -> &[u8] {
    let mut last = s.len();
    while last > 0 {
        let mut adjust_flag = false;
        if let Some(c) = char::from_u32(s[last - 1] as u32) {
            if c.is_ascii_whitespace() {
                last -= 1;
                adjust_flag = true;
            }
        }
        if !adjust_flag {
            break;
        }
    }
    &s[0..last]
}

/// Default comparer which compare two files/strings/bytes line by line
/// and ignore white space at line end and empty line at file end.
impl Default for Comparer {
    fn default() -> Comparer {
        Comparer::new(true, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures::executor::block_on;
    use tempfile;
    use tokio::fs;

    fn diff(output: &[u8], answer: &[u8]) -> bool {
        !block_on(Comparer::default().compare(&output, &answer))
    }

    #[tokio::test]
    async fn test_diff_complete_eq() {
        assert!(!diff(b"hello world", b"hello world"));
    }

    #[tokio::test]
    async fn test_diff_with_empty_line_at_eof() {
        assert!(!diff(b"hello world\n", b"hello world"));
        assert!(!diff(b"hello world\n\n", b"hello world"));
        assert!(!diff(b"hello world\n", b"hello world\n"));
        assert!(!diff(b"hello world\n\n", b"hello world\n"));
        assert!(!diff(b"hello world\n\n", b"hello world\n\n"));
        assert!(!diff(b"hello world", b"hello world\n"));
        assert!(!diff(b"hello world", b"hello world\n\n"));
    }

    #[tokio::test]
    async fn test_diff_with_space_at_eol() {
        assert!(!diff(b"hello world ", b"hello world"));
        assert!(!diff(b"hello world  ", b"hello world"));
        assert!(!diff(b"hello world", b"hello world  "));
    }

    #[tokio::test]
    async fn test_diff_with_both_empty_line_at_eof_and_space_at_eol() {
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

    #[tokio::test]
    async fn test_check() -> io::Result<()> {
        let work_dir = tempfile::tempdir()?;
        let file0 = work_dir.path().join("test_check_without_spj.0");
        let file1 = work_dir.path().join("test_check_without_spj.1");
        let file2 = work_dir.path().join("test_check_without_spj.2");
        fs::write(&file0, "hello world").await?;
        fs::write(&file1, "hello world").await?;
        fs::write(&file2, "hello_world").await?;

        let comparer = Comparer::default();
        assert!(comparer.compare_files(&file0, &file1).await?);
        assert!(!comparer.compare_files(&file0, &file2).await?);

        Ok(())
    }
}
