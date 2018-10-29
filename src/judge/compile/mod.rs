mod compilers;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use self::compilers::Compiler;
pub use self::compilers::Languages;

type CompileResult = Result<(), ()>;

pub fn compile(language: &Languages, source_code: &str, executable_file: &Path) -> CompileResult {
    match language {
        Languages::CGcc => {
            use self::compilers::c_gcc::CGcc;
            _compile(CGcc {}, source_code, executable_file)
        }
        Languages::CppGxx => {
            use self::compilers::cpp_gxx::CppGxx;
            _compile(CppGxx {}, source_code, executable_file)
        }
    }
}

fn _compile(compiler: impl Compiler, source_code: &str, executable_file: &Path) -> CompileResult {
    let mut source_file_path = env::temp_dir();
    source_file_path.push("main");
    source_file_path.set_extension(compiler.suffix());

    let mut source_file = File::create(source_file_path).expect("Cannot create source file");
    source_file
        .write(source_code.as_bytes())
        .expect("Cannot write source code to file");
    source_file
        .flush()
        .expect("Cannot write source code to file");
    drop(source_file);

    let mut source_file_path = env::temp_dir();
    source_file_path.push("main");
    source_file_path.set_extension(compiler.suffix());

    compiler.compile(source_file_path.as_path(), executable_file, true)
}
