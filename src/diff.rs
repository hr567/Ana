use std::fs;
use std::io::{self, BufRead};
use std::path;
use std::process;

/// Check if two buffers is equal
///
/// Ignore the empty line at end of file and space at end of line.
/// Return `false` if there is no difference between two buffer
fn buf_diff<T: BufRead, U: BufRead>(output_buf: &mut T, answer_buf: &mut U) -> io::Result<bool> {
    loop {
        let (output, output_eof) = {
            let mut res = Vec::new();
            let eof = output_buf.read_until(b'\n', &mut res)? == 0;
            while res.ends_with(&[b' ']) || res.ends_with(&[b'\n']) {
                res.pop();
            }
            (res, eof)
        };

        let (answer, answer_eof) = {
            let mut res = Vec::new();
            let eof = answer_buf.read_until(b'\n', &mut res)? == 0;
            while res.ends_with(&[b' ']) || res.ends_with(&[b'\n']) {
                res.pop();
            }
            (res, eof)
        };

        if output_eof && answer_eof {
            break;
        }

        if output != answer {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Compare output file and answer file line by line
///
/// Ignore the blank line at the end of file
/// and white space at the end of line.
/// Return `true` if there is no difference between two files
///
/// Return `Err` when appear any IO error
pub fn check(output_file: &path::Path, answer_file: &path::Path) -> io::Result<bool> {
    let mut output_buf = {
        let file = fs::File::open(output_file)?;
        io::BufReader::new(file)
    };

    let mut answer_buf = {
        let file = fs::File::open(answer_file)?;
        io::BufReader::new(file)
    };

    Ok(!buf_diff(&mut output_buf, &mut answer_buf)?)
}

/// Check output by provided spj program
///
/// Return `true` if spj program exit successfully
pub fn check_with_spj(
    input_file: &path::Path,
    output_file: &path::Path,
    answer_file: &path::Path,
    spj: &path::Path,
) -> io::Result<bool> {
    Ok(process::Command::new(spj)
        .arg(input_file)
        .arg(answer_file)
        .arg(output_file)
        .status()?
        .success())
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile;

    fn diff(output: &[u8], answer: &[u8]) -> bool {
        buf_diff(&mut io::Cursor::new(output), &mut io::Cursor::new(answer)).unwrap()
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
        assert!(check(&file0, &file1)?);
        assert!(!check(&file0, &file2)?);
        Ok(())
    }
}
