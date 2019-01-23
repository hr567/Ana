use std::io;
use std::path::Path;
use std::process::Command;

use super::*;

pub struct CppGxx;

impl CppGxx {
    pub fn new() -> CppGxx {
        CppGxx {}
    }
}

impl Compiler for CppGxx {
    fn suffix(&self) -> &'static str {
        "cpp"
    }

    fn compile(&self, source_file: &Path, executable_file: &Path) -> io::Result<Result> {
        let source_file = rename_with_new_extension(&source_file, self.suffix())
            .expect("Failed to rename source file");
        let res = Command::new("g++")
            .arg(source_file.to_str().unwrap())
            .args(&["-o", executable_file.to_str().unwrap()])
            .arg("-O2")
            .arg("-fno-asm")
            .arg("-Wall")
            .arg("-lm")
            .arg("--static")
            .arg("--std=c++11")
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
        let source_file_path = work_dir.path().join("cpp_compiler_test_pass.cpp");
        fs::write(
            &source_file_path,
            "#include<iostream>\nint main() { return 0; }",
        )?;
        let executable_file_path = work_dir.path().join("cpp_compiler_test_pass.exe");
        assert!(CppGxx::new()
            .compile(&source_file_path, &executable_file_path)?
            .is_ok());
        Ok(())
    }

    #[test]
    fn test_compile_failed() -> io::Result<()> {
        let work_dir = tempfile::tempdir()?;
        let source_file_path = work_dir.path().join("cpp_compiler_test_fail.cpp");
        fs::write(
            &source_file_path,
            "#include<iostream>\nint main() { return 0 }",
        )?;
        let executable_file_path = work_dir.path().join("cpp_compiler_test_fail.exe");
        assert!(CppGxx::new()
            .compile(&source_file_path, &executable_file_path)?
            .is_err());
        Ok(())
    }
}
