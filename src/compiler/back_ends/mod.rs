mod c_gcc;
mod cpp_gxx;

use super::{CompileResult, Compiler};

pub use self::{c_gcc::CGcc, cpp_gxx::CppGxx};
