use std::path::Path;

use super::Compiler;
use std::process::Command;

pub struct CGcc {}

impl Compiler for CGcc {
    fn suffix(&self) -> &'static str {
        &"c"
    }

    fn compile(
        &self,
        source_file: &Path,
        executable_file: &Path,
        optimize_flag: bool,
    ) -> Result<(), ()> {
        let status = Command::new("gcc")
            .arg(source_file.to_str().unwrap())
            .args(&["-o", executable_file.to_str().unwrap()])
            .arg(if optimize_flag { "-O2" } else { "" })
            .arg("-fno-asm")
            .arg("-Wall")
            .arg("-lm")
            .arg("--static")
            .arg("--std=c99")
            .status()
            .expect("Failed to compile the source");
        if status.success() {
            Ok(())
        } else {
            Err(())
        }
    }
}
