use std::env;
use std::fs;
use std::io;
use std::path;

mod back_ends;

use self::back_ends::{CGcc, CppGxx};

pub struct Compiler;

impl Compiler {
    pub fn compile(
        language: &str,
        source_code: &str,
        executable_file: &path::Path,
    ) -> io::Result<bool> {
        match language {
            "c.gcc" => {
                let source_file = generate_source_file(source_code, <Compiler as CGcc>::suffix());
                <Compiler as CGcc>::compile(&source_file?, executable_file)
            }
            "cpp.gxx" => {
                let source_file = generate_source_file(source_code, <Compiler as CppGxx>::suffix());
                <Compiler as CppGxx>::compile(&source_file?, executable_file)
            }
            _ => unimplemented!("Language or compiler {} is not support", language),
        }
    }
}

fn generate_source_file(source_code: &str, suffix: &str) -> io::Result<Box<path::Path>> {
    let mut source_file = path::PathBuf::from(env::var("ANA_WORK_DIR").unwrap());
    source_file.push("main");
    source_file.set_extension(suffix);
    fs::write(source_file.as_path(), source_code.as_bytes())?;

    Ok(source_file.into_boxed_path())
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;

    #[test]
    fn test_c_language_compile() -> io::Result<()> {
        env::set_var("ANA_WORK_DIR", env::temp_dir());
        let executable_file_path = env::temp_dir().join("c_compile_test.exe");
        assert!(Compiler::compile(
            "c.gcc",
            "#include<stdio.h>\nint main() { return 0; }",
            &executable_file_path,
        )?);
        assert!(Command::new(&executable_file_path).status()?.success());
        Ok(())
    }

    #[test]
    fn test_cpp_language_compile() -> io::Result<()> {
        env::set_var("ANA_WORK_DIR", env::temp_dir());
        let executable_file_path = env::temp_dir().join("cpp_compile_test.exe");
        assert!(Compiler::compile(
            "cpp.gxx",
            "#include<iostream>\nint main() { return 0; }",
            &executable_file_path,
        )?);
        assert!(Command::new(&executable_file_path).status()?.success());
        Ok(())
    }
}
