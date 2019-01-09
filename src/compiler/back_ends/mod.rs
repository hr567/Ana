mod c_gcc;
mod cpp_gxx;

use super::{rename_with_new_extension, Compiler};

pub use self::{c_gcc::CGcc, cpp_gxx::CppGxx};
