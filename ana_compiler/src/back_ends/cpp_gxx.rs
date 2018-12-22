use std::io;
use std::path::Path;
use std::process::Command;

use super::Compiler;

pub trait CppGxx {
    fn suffix() -> &'static str {
        "cpp"
    }
    fn compile(source_file: &Path, executable_file: &Path) -> io::Result<bool>;
}

impl CppGxx for Compiler {
    fn compile(source_file: &Path, executable_file: &Path) -> io::Result<bool> {
        Ok(Command::new("g++")
            .arg(source_file.to_str().unwrap())
            .args(&["-o", executable_file.to_str().unwrap()])
            .arg("-O2")
            .arg("-fno-asm")
            .arg("-Wall")
            .arg("-lm")
            .arg("--static")
            .arg("--std=c++11")
            .status()?
            .success())
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::io;

    use super::*;

    #[test]
    fn test_suffix() {
        assert_eq!(<Compiler as CppGxx>::suffix(), "cpp");
    }

    #[test]
    fn test_compile() -> io::Result<()> {
        let source_file_path = env::temp_dir().join("cpp_compiler_test_pass.cpp");
        fs::write(
            &source_file_path,
            "#include<iostream>\nint main() { return 0; }",
        )?;
        let executable_file_path = env::temp_dir().join("cpp_compiler_test_pass.exe");
        assert!(<Compiler as CppGxx>::compile(
            &source_file_path,
            &executable_file_path,
        )?);
        Ok(())
    }

    #[test]
    fn test_compile_failed() -> io::Result<()> {
        let source_file_path = env::temp_dir().join("cpp_compiler_test_fail.cpp");
        fs::write(
            &source_file_path,
            "#include<iostream>\nint main() { return 0 }",
        )?;
        let executable_file_path = env::temp_dir().join("cpp_compiler_test_fail.exe");
        assert!(!<Compiler as CppGxx>::compile(
            &source_file_path,
            &executable_file_path
        )?);
        Ok(())
    }
}
