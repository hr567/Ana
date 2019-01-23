use std::fs;
use std::io;
use std::path;
use std::result;

mod back_ends;
use back_ends::get_compiler;

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

pub trait Compiler {
    fn suffix(&self) -> &'static str;
    fn compile(&self, source_file: &path::Path, executable_file: &path::Path)
        -> io::Result<Result>;
}
