use std::path;
use std::process;

use super::*;

pub struct CGcc;

impl CGcc {
    pub fn new() -> CGcc {
        CGcc {}
    }
}

impl Compiler for CGcc {
    fn compile(&self, source_file: &path::Path, executable_file: &path::Path) -> CompileFuture {
        let source_file =
            rename_with_new_extension(&source_file, "c").expect("Failed to rename source file");
        process::Command::new("gcc")
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
            .arg("--std=c99")
            .spawn()
            .expect("Failed to run gcc")
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    use tempfile;
    use tokio_threadpool;

    #[test]
    fn test_c_gcc_compile() {
        let work_dir = tempfile::tempdir().unwrap();
        let source_file = work_dir.path().join("c_compiler_test_pass.c");
        let executable_file = work_dir.path().join("c_compiler_test_pass.exe");
        fs::write(&source_file, "#include<stdio.h>\nint main() { return 0; }").unwrap();
        let pool = tokio_threadpool::ThreadPool::new();
        let compile_result = pool
            .spawn_handle(CGcc::new().compile(&source_file, &executable_file))
            .wait()
            .unwrap();
        assert!(compile_result);
    }

    #[test]
    fn test_c_gcc_compile_failed() {
        let work_dir = tempfile::tempdir().unwrap();
        let source_file = work_dir.path().join("c_compiler_test_fail.c");
        let executable_file = work_dir.path().join("c_compiler_test_fail.exe");
        fs::write(&source_file, "#include<stdio.h>\nint main() { return 0 }").unwrap();
        let pool = tokio_threadpool::ThreadPool::new();
        let compile_result = pool
            .spawn_handle(CGcc::new().compile(&source_file, &executable_file))
            .wait()
            .unwrap();
        assert!(!compile_result);
    }
}
