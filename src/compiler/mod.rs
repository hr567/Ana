/// Interface for different compilers
use std::path;

mod back_ends;
use back_ends::get_compiler;

pub fn compile(language: &str, source_file: &path::Path, executable_file: &path::Path) -> bool {
    if let Ok(compiler) = get_compiler(&language) {
        compiler.compile(&source_file, &executable_file)
    } else {
        unimplemented!("Unsupported language or compiler")
    }
}

pub trait Compiler {
    fn compile(&self, source_file: &path::Path, executable_file: &path::Path) -> bool;
}
