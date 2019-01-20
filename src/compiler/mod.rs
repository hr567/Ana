use std::fs;
use std::io;
use std::path;
use std::result;

mod back_ends;

use self::back_ends::{CGcc, CppGxx};

pub type Result = result::Result<(), String>;

pub fn compile(
    language: &str,
    source_file: &path::Path,
    executable_file: &path::Path,
) -> io::Result<Result> {
    if let Ok(compiler) = get_compiler(&language) {
        compiler.compile(&source_file, &executable_file)
    } else {
        unimplemented!("Unsupported language or compiler")
    }
}

trait Compiler {
    fn suffix(&self) -> &'static str;
    fn compile(&self, source_file: &path::Path, executable_file: &path::Path)
        -> io::Result<Result>;
}

fn get_compiler(language: &str) -> result::Result<Box<dyn Compiler>, &'static str> {
    match language {
        "c.gcc" => Ok(Box::new(CGcc::new())),
        "cpp.gxx" => Ok(Box::new(CppGxx::new())),
        _ => Err("Language or compiler is not support"),
    }
}

fn rename_with_new_extension(
    origin_file: &path::Path,
    new_extension: &str,
) -> io::Result<Box<path::Path>> {
    let mut new_file = origin_file.to_path_buf();
    new_file.set_extension(new_extension);
    fs::rename(&origin_file, &new_file)?;
    Ok(new_file.into_boxed_path())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;

    use tempfile;

    #[test]
    fn test_c_language_compile() -> io::Result<()> {
        let work_dir = tempfile::tempdir()?;
        let executable_file = work_dir.path().join("c_compile_test.exe");
        let source_file = work_dir.path().join("c_compile_test.c");
        fs::write(&source_file, "#include<stdio.h>\nint main() { return 0; }")
            .expect("Failed to write source code");
        let compiler = get_compiler("c.gcc").unwrap();
        assert!(compiler.compile(&source_file, &executable_file)?.is_ok());
        assert!(process::Command::new(&executable_file).status()?.success());
        Ok(())
    }

    #[test]
    fn test_cpp_language_compile() -> io::Result<()> {
        let work_dir = tempfile::tempdir()?;
        let executable_file = work_dir.path().join("cpp_compile_test.exe");
        let source_file = work_dir.path().join("cpp_compile_test.cpp");
        fs::write(&source_file, "#include<iostream>\nint main() { return 0; }")
            .expect("Failed to write source code");
        let compiler = get_compiler("cpp.gxx").unwrap();
        assert!(compiler.compile(&source_file, &executable_file)?.is_ok());
        assert!(process::Command::new(&executable_file).status()?.success());
        Ok(())
    }
}
