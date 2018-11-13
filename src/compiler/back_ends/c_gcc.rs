use std::path::Path;
use std::process::Command;

use super::{CompileResult, Compiler};

pub trait CGcc {
    fn suffix() -> &'static str {
        "c"
    }
    fn compile(source_file: &Path, executable_file: &Path) -> CompileResult;
}

impl CGcc for Compiler {
    fn compile(source_file: &Path, executable_file: &Path) -> CompileResult {
        let status = Command::new("gcc")
            .arg(source_file.to_str().unwrap())
            .args(&["-o", executable_file.to_str().unwrap()])
            .arg("-O2")
            .arg("-fno-asm")
            .arg("-Wall")
            .arg("-lm")
            .arg("--static")
            .arg("--std=c99")
            .status()
            .expect("Failed to compile the source");
        if status.success() {
            CompileResult::Pass
        } else {
            CompileResult::CE
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
        assert_eq!(<Compiler as CGcc>::suffix(), "c");
    }

    #[test]
    fn test_compile() {
        let mut source_file_path = env::temp_dir();
        source_file_path.push("c_compiler_test_pass.c");

        let mut source_file = File::create(source_file_path.as_path()).unwrap();
        source_file.write(b"int main() { return 0; }\n\n").unwrap();
        source_file.flush().unwrap();

        let mut executable_file_path = env::temp_dir();
        executable_file_path.push("c_compiler_test_pass.exe");

        assert_eq!(
            <Compiler as CGcc>::compile(
                source_file_path.as_path(),
                executable_file_path.as_path(),
            ),
            CompileResult::Pass
        );
    }

    #[test]
    fn test_compile_failed() {
        let mut source_file_path = env::temp_dir();
        source_file_path.push("c_compiler_test_fail.c");

        let mut source_file = File::create(source_file_path.as_path()).unwrap();
        source_file.write(b"int main() { return 0 }\n\n").unwrap();
        source_file.flush().unwrap();

        let mut executable_file_path = env::temp_dir();
        executable_file_path.push("c_compiler_test_fail.exe");

        assert_eq!(
            <Compiler as CGcc>::compile(
                source_file_path.as_path(),
                executable_file_path.as_path(),
            ),
            CompileResult::CE
        );
    }
}
