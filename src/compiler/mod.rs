/// Interface for different compilers
use std::path;

mod back_ends;
use back_ends::get_compiler;

/// Compile the source file to the executable file
pub fn compile(language: &str, source_file: &path::Path, executable_file: &path::Path) -> bool {
    if let Ok(compile) = get_compiler(&language) {
        compile(&source_file, &executable_file)
    } else {
        unimplemented!("Unsupported language or compiler")
    }
}

pub trait Compiler {
    const SOURCE_SUFFIX: &'static str;

    fn compile(source_file: &path::Path, executable_file: &path::Path) -> bool;
}
