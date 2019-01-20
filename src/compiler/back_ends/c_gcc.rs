use std::io;
use std::path::Path;
use std::process;

use super::*;

pub struct CGcc;

impl CGcc {
    pub fn new() -> CGcc {
        CGcc {}
    }
}

impl Compiler for CGcc {
    fn suffix(&self) -> &'static str {
        "c"
    }

    fn compile(&self, source_file: &Path, executable_file: &Path) -> io::Result<Result> {
        let source_file = rename_with_new_extension(&source_file, self.suffix())
            .expect("Failed to rename source file");
        let res = process::Command::new("gcc")
            .stdin(process::Stdio::null())
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::piped())
            .arg(source_file.to_str().unwrap())
            .args(&["-o", executable_file.to_str().unwrap()])
            .arg("-O2")
            .arg("-fno-asm")
            .arg("-Wall")
            .arg("-lm")
            .arg("--static")
            .arg("--std=c99")
            .output()?;
        Ok(if res.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8(res.stderr).unwrap_or_default())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io;

    use tempfile;

    #[test]
    fn test_compile() -> io::Result<()> {
        let work_dir = tempfile::tempdir()?;
        let source_file_path = work_dir.path().join("c_compiler_test_pass.c");
        fs::write(
            &source_file_path,
            "#include<stdio.h>\nint main() { return 0; }",
        )?;
        let executable_file_path = work_dir.path().join("c_compiler_test_pass.exe");
        assert!(CGcc::new()
            .compile(&source_file_path, &executable_file_path)?
            .is_ok());
        Ok(())
    }

    #[test]
    fn test_compile_failed() -> io::Result<()> {
        let work_dir = tempfile::tempdir()?;
        let source_file_path = work_dir.path().join("c_compiler_test_fail.c");
        fs::write(
            &source_file_path,
            "#include<stdio.h>\nint main() { return 0 }",
        )?;
        let executable_file_path = work_dir.path().join("c_compiler_test_fail.exe");
        assert!(CGcc::new()
            .compile(&source_file_path, &executable_file_path)?
            .is_err());
        Ok(())
    }
}
