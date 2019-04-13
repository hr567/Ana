use std::path;
use std::process;
use std::thread;
use std::time;

use super::*;

pub struct CppGxx;

impl CppGxx {
    pub fn new() -> CppGxx {
        CppGxx {}
    }
}

impl Compiler for CppGxx {
    fn compile(&self, source_file: &path::Path, executable_file: &path::Path) -> bool {
        let source_file =
            rename_with_new_extension(&source_file, "cpp").expect("Failed to rename source file");
        let mut child = process::Command::new("g++")
            .stdin(process::Stdio::null())
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::piped())
            .arg(source_file.as_os_str())
            .arg("-o")
            .arg(executable_file.as_os_str())
            .arg("-O2")
            .arg("-fno-asm")
            .arg("-Wall")
            .arg("-lm")
            .arg("--static")
            .arg("--std=c++11")
            .spawn()
            .expect("Failed to run g++");
        let mut compile_success = false;
        let start_compiling_time = time::Instant::now();
        while start_compiling_time.elapsed() < time::Duration::from_secs(10) {
            match child.try_wait() {
                Ok(Some(status)) => {
                    compile_success = status.success();
                    break;
                }
                Ok(None) => thread::sleep(time::Duration::from_millis(500)),
                Err(e) => panic!("Error attempting to wait: {}", e),
            }
        }
        compile_success
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    use tempfile;

    #[test]
    fn test_cpp_gxx_compile() {
        let work_dir = tempfile::tempdir().unwrap();
        let source_file = work_dir.path().join("cpp_compiler_test_pass.cpp");
        let executable_file = work_dir.path().join("cpp_compiler_test_pass.exe");
        fs::write(&source_file, "#include<iostream>\nint main() { return 0; }").unwrap();
        let compile_success = CppGxx::new().compile(&source_file, &executable_file);
        assert!(compile_success);
    }

    #[test]
    fn test_cpp_gxx_compile_failed() {
        let work_dir = tempfile::tempdir().unwrap();
        let source_file = work_dir.path().join("cpp_compiler_test_fail.cpp");
        let executable_file = work_dir.path().join("cpp_compiler_test_fail.exe");
        fs::write(&source_file, "#include<iostream>\nint main() { return 0 }").unwrap();
        let compile_success = CppGxx::new().compile(&source_file, &executable_file);
        assert!(!compile_success);
    }
}
