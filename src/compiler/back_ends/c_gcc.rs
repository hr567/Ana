use std::path;
use std::process;
use std::thread;
use std::time;

use super::*;

pub struct CGcc;

impl CGcc {
    pub fn new() -> CGcc {
        CGcc {}
    }
}

impl Compiler for CGcc {
    fn compile(&self, source_file: &path::Path, executable_file: &path::Path) -> bool {
        let source_file =
            rename_with_new_extension(&source_file, "c").expect("Failed to rename source file");
        let mut child = process::Command::new("gcc")
            .stdin(process::Stdio::null())
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::inherit())
            .arg(source_file.as_os_str())
            .arg("-o")
            .arg(executable_file.as_os_str())
            .arg("-O2")
            .arg("-fno-asm")
            .arg("-Wall")
            .arg("-lm")
            .arg("--static")
            .arg("--std=c99")
            .spawn()
            .expect("Failed to run gcc");
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
    fn test_c_gcc_compile() {
        let work_dir = tempfile::tempdir().unwrap();
        let source_file = work_dir.path().join("c_compiler_test_pass.c");
        let executable_file = work_dir.path().join("c_compiler_test_pass.exe");
        fs::write(&source_file, "#include<stdio.h>\nint main() { return 0; }").unwrap();
        let compile_success = CGcc::new().compile(&source_file, &executable_file);
        assert!(compile_success);
    }

    #[test]
    fn test_c_gcc_compile_failed() {
        let work_dir = tempfile::tempdir().unwrap();
        let source_file = work_dir.path().join("c_compiler_test_fail.c");
        let executable_file = work_dir.path().join("c_compiler_test_fail.exe");
        fs::write(&source_file, "#include<stdio.h>\nint main() { return 0 }").unwrap();
        let compile_success = CGcc::new().compile(&source_file, &executable_file);
        assert!(!compile_success);
    }
}
