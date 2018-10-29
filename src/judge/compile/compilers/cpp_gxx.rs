use std::path::Path;

use super::Compiler;
use std::process::Command;

pub struct CppGxx {}

impl Compiler for CppGxx {
    fn suffix(&self) -> &'static str {
        &"cpp"
    }

    fn compile(
        &self,
        source_file: &Path,
        executable_file: &Path,
        optimize_flag: bool,
    ) -> Result<(), ()> {
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
