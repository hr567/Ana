mod compilers;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub use self::compilers::Languages;
use self::compilers::{CompileResult, Compiler};

pub fn compile(language: &Languages, source_code: &str, executable_file: &Path) -> CompileResult {
    match language {
        Languages::CGcc => {
            use self::compilers::c_gcc::CGcc;
            _compile(CGcc::new(), source_code, executable_file)
        }
        Languages::CppGxx => {
            use self::compilers::cpp_gxx::CppGxx;
            _compile(CppGxx::new(), source_code, executable_file)
        }
    }
}

fn _compile(compiler: impl Compiler, source_code: &str, executable_file: &Path) -> CompileResult {
    let mut source_file_path = env::temp_dir();
    source_file_path.push("main");
    source_file_path.set_extension(compiler.suffix());

    let mut source_file =
        File::create(source_file_path.as_path()).expect("Cannot create source file");
    source_file
        .write(source_code.as_bytes())
        .expect("Cannot write source code to file");
    source_file
        .flush()
        .expect("Cannot write source code to file");

    compiler.compile(source_file_path.as_path(), executable_file, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::env;
    use std::process::Command;

    #[test]
    fn test_c_language_compile() {
        let mut executable_file_path = env::temp_dir();
        executable_file_path.push("c_language_compile_test.exe");
        let compile_result = compile(
            &Languages::CGcc,
            "int main() { return 0; }\n\n",
            executable_file_path.as_path(),
        );
        assert_eq!(compile_result, Ok(()));

        let exit_status = Command::new(executable_file_path.to_str().unwrap())
            .status()
            .unwrap();
        assert!(exit_status.success());
    }

    #[test]
    fn test_cpp_language_compile() {
        let mut executable_file_path = env::temp_dir();
        executable_file_path.push("cpp_language_compile_test.exe");
        let compile_result = compile(
            &Languages::CppGxx,
            "int main() { return 0; }\n\n",
            executable_file_path.as_path(),
        );
        assert_eq!(compile_result, Ok(()));

        let exit_status = Command::new(executable_file_path.to_str().unwrap())
            .status()
            .unwrap();
        assert!(exit_status.success());
    }

    #[test]
    fn test_c_compiler_compile() {
        let mut executable_file_path = env::temp_dir();
        executable_file_path.push("c_compiler_compile_test.exe");
        let compile_result = _compile(
            compilers::c_gcc::CGcc::new(),
            "int main() { return 0; }\n\n",
            executable_file_path.as_path(),
        );
        assert_eq!(compile_result, Ok(()));

        let exit_status = Command::new(executable_file_path.to_str().unwrap())
            .status()
            .unwrap();
        assert!(exit_status.success());
    }

    #[test]
    fn test_cpp_compiler_compile() {
        let mut executable_file_path = env::temp_dir();
        executable_file_path.push("cpp_compiler_compile_test.exe");
        let compile_result = _compile(
            compilers::cpp_gxx::CppGxx::new(),
            "int main() { return 0; }\n\n",
            executable_file_path.as_path(),
        );
        assert_eq!(compile_result, Ok(()));

        let exit_status = Command::new(executable_file_path.to_str().unwrap())
            .status()
            .unwrap();
        assert!(exit_status.success());
    }
}
