pub mod c_gcc;
pub mod cpp_gxx;

use std::path::Path;

pub type CompileResult = Result<(), ()>;

pub enum Languages {
    CGcc,
    CppGxx,
}

pub trait Compiler {
    fn suffix(&self) -> &'static str;
    fn compile(
        &self,
        source_file: &Path,
        executable_file: &Path,
        optimize_flag: bool,
    ) -> CompileResult;
}
