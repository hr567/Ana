use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path;

mod back_ends;

use self::back_ends::{CGcc, CppGxx};

pub struct Compiler {}

impl Compiler {
    pub fn compile(
        language: &str,
        source_code: &str,
        executable_file: &path::Path,
    ) -> Result<(), ()> {
        match language {
            "c.gcc" => {
                let source_file = generate_source_file(source_code, <Compiler as CGcc>::suffix());
                <Compiler as CGcc>::compile(source_file.as_ref(), executable_file)
            }
            "cpp.gxx" => {
                let source_file = generate_source_file(source_code, <Compiler as CppGxx>::suffix());
                <Compiler as CppGxx>::compile(source_file.as_ref(), executable_file)
            }
            _ => unimplemented!("Language or compiler {} is not support", language),
        }
    }
}

fn generate_source_file(source_code: &str, suffix: &str) -> Box<path::Path> {
    let mut source_file = path::PathBuf::from(env::var("ANA_WORK_DIR").unwrap());
    source_file.push("main");
    source_file.set_extension(suffix);
    File::create(source_file.as_path())
        .expect("Cannot create source file")
        .write_all(source_code.as_bytes())
        .expect("Failed to write source code to source file");
    source_file.into_boxed_path()
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;

    #[test]
    fn test_c_language_compile() {
        env::set_var("ANA_WORK_DIR", env::temp_dir());
        let mut executable_file_path = path::PathBuf::from(env::var("ANA_WORK_DIR").unwrap());
        executable_file_path.push("c_compile_test");
        executable_file_path.set_extension("exe");
        let compile_result = Compiler::compile(
            "c.gcc",
            "int main() { return 0; }",
            executable_file_path.as_path(),
        );
        assert!(compile_result.is_ok());

        let exit_status = Command::new(executable_file_path).status().unwrap();
        assert!(exit_status.success());
    }

    #[test]
    fn test_cpp_language_compile() {
        env::set_var("ANA_WORK_DIR", env::temp_dir());
        let mut executable_file_path = path::PathBuf::from(env::var("ANA_WORK_DIR").unwrap());
        executable_file_path.push("cpp_compile_test");
        executable_file_path.set_extension("exe");
        let compile_result = Compiler::compile(
            "cpp.gxx",
            "int main() { return 0; }",
            executable_file_path.as_path(),
        );
        assert!(compile_result.is_ok());

        let exit_status = Command::new(executable_file_path).status().unwrap();
        assert!(exit_status.success());
    }
}
