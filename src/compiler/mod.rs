use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

mod back_ends;

use self::back_ends::{CGcc, CppGxx};

#[derive(PartialEq, Debug)]
pub enum CompileResult {
    Pass,
    CE,
}

pub struct Compiler {}

impl Compiler {
    pub fn compile(language: &str, source_code: &str, executable_file: &Path) -> CompileResult {
        match language {
            "c.gcc" => {
                let source_file = generate_source_file(source_code, <Compiler as CGcc>::suffix());
                <Compiler as CGcc>::compile(source_file.as_path(), executable_file)
            }
            "cpp.gxx" => {
                let source_file = generate_source_file(source_code, <Compiler as CppGxx>::suffix());
                <Compiler as CppGxx>::compile(source_file.as_path(), executable_file)
            }
            _ => unimplemented!("Language or compiler {} is not support", language),
        }
    }
}

fn generate_source_file(source_code: &str, suffix: &str) -> PathBuf {
    let mut source_file = env::temp_dir();
    source_file.push("main");
    source_file.set_extension(suffix);
    File::create(source_file.as_path())
        .expect("Cannot create source file")
        .write_all(source_code.as_bytes())
        .expect("Failed to write source code to source file");
    source_file
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::env;
    use std::process::Command;

    #[test]
    fn test_c_language_compile() {
        let mut executable_file_path = env::temp_dir();
        executable_file_path.push("c_compile_test.exe");
        let compile_result = Compiler::compile(
            &"c.gcc",
            "int main() { return 0; }\n\n",
            executable_file_path.as_path(),
        );
        assert_eq!(compile_result, CompileResult::Pass);

        let exit_status = Command::new(executable_file_path.to_str().unwrap())
            .status()
            .unwrap();
        assert!(exit_status.success());
    }

    #[test]
    fn test_cpp_language_compile() {
        let mut executable_file_path = env::temp_dir();
        executable_file_path.push("cpp_compile_test.exe");
        let compile_result = Compiler::compile(
            &"cpp.gxx",
            "int main() { return 0; }\n\n",
            executable_file_path.as_path(),
        );
        assert_eq!(compile_result, CompileResult::Pass);

        let exit_status = Command::new(executable_file_path.to_str().unwrap())
            .status()
            .unwrap();
        assert!(exit_status.success());
    }
}
