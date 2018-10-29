use std::path::Path;

use super::{CompileResult, Compiler};
use std::process::Command;

pub struct CppGxx {}

impl CppGxx {
    pub fn new() -> Self {
        CppGxx {}
    }
}

impl Compiler for CppGxx {
    fn suffix(&self) -> &'static str {
        &"cpp"
    }

    fn compile(
        &self,
        source_file: &Path,
        executable_file: &Path,
        optimize_flag: bool,
    ) -> CompileResult {
        let status = Command::new("g++")
            .arg(source_file.to_str().unwrap())
            .args(&["-o", executable_file.to_str().unwrap()])
            .arg(if optimize_flag { "-O2" } else { "" })
            .arg("-fno-asm")
            .arg("-Wall")
            .arg("-lm")
            .arg("--static")
            .arg("--std=c++11")
            .status()
            .expect("Failed to compile the source");
        if status.success() {
            Ok(())
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs::File;
    use std::io::prelude::*;

    use super::*;

    #[test]
    fn test_suffix() {
        let compiler = CppGxx::new();
        assert_eq!(compiler.suffix(), "cpp");
    }

    #[test]
    fn test_compile() {
        let compiler = CppGxx::new();

        let mut source_file_path = env::temp_dir();
        source_file_path.push("cpp_compiler_test_pass.c");

        println!("{}", source_file_path.to_str().unwrap());
        let mut source_file = File::create(source_file_path.as_path()).unwrap();
        source_file.write(b"int main() { return 0; }\n\n").unwrap();
        source_file.flush().unwrap();

        let mut executable_file_path = env::temp_dir();
        executable_file_path.push("cpp_compiler_test_pass.exe");

        assert_eq!(
            compiler.compile(
                source_file_path.as_path(),
                executable_file_path.as_path(),
                true,
            ),
            Ok(())
        );
    }

    #[test]
    fn test_compile_failed() {
        let compiler = CppGxx::new();

        let mut source_file_path = env::temp_dir();
        source_file_path.push("cpp_compiler_test_fail.c");

        let mut source_file = File::create(source_file_path.as_path()).unwrap();
        source_file.write(b"int main() { return 0 }\n\n").unwrap();
        source_file.flush().unwrap();

        let mut executable_file_path = env::temp_dir();
        executable_file_path.push("cpp_compiler_test_fail.exe");

        assert_eq!(
            compiler.compile(
                source_file_path.as_path(),
                executable_file_path.as_path(),
                true,
            ),
            Err(())
        );
    }
}
