use std::io;
use std::path::Path;
use std::process::Command;

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

    fn compile(&self, source_file: &Path, executable_file: &Path) -> io::Result<bool> {
        let source_file = rename_with_new_extension(&source_file, self.suffix())
            .expect("Failed to rename source file");
        Ok(Command::new("gcc")
            .arg(source_file.to_str().unwrap())
            .args(&["-o", executable_file.to_str().unwrap()])
            .arg("-O2")
            .arg("-fno-asm")
            .arg("-Wall")
            .arg("-lm")
            .arg("--static")
            .arg("--std=c99")
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
    fn test_compile() -> io::Result<()> {
        let source_file_path = env::temp_dir().join("c_compiler_test_pass.c");
        fs::write(
            &source_file_path,
            "#include<stdio.h>\nint main() { return 0; }",
        )?;
        let executable_file_path = env::temp_dir().join("c_compiler_test_pass.exe");
        assert!(CGcc::new().compile(&source_file_path, &executable_file_path)?);
        Ok(())
    }

    #[test]
    fn test_compile_failed() -> io::Result<()> {
        let source_file_path = env::temp_dir().join("c_compiler_test_fail.c");
        fs::write(
            &source_file_path,
            "#include<stdio.h>\nint main() { return 0 }",
        )?;
        let executable_file_path = env::temp_dir().join("c_compiler_test_fail.exe");
        assert!(!CGcc::new().compile(&source_file_path, &executable_file_path)?);
        Ok(())
    }
}
