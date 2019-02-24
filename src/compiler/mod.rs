/// Interface for different compilers
use std::fs;
use std::io;
use std::path;
use std::process;
use std::str;

use tokio::prelude::*;
use tokio_threadpool;

mod back_ends;
use back_ends::get_compiler;

pub fn compile(
    language: &str,
    source_file: &path::Path,
    executable_file: &path::Path,
) -> impl Future<Item = bool, Error = ()> {
    if let Ok(compiler) = get_compiler(&language) {
        compiler.compile(&source_file, &executable_file)
    } else {
        unimplemented!("Unsupported language or compiler")
    }
}

pub trait Compiler {
    fn compile(&self, source_file: &path::Path, executable_file: &path::Path) -> CompileFuture;
}

pub struct CompileFuture(process::Child);

impl From<process::Child> for CompileFuture {
    fn from(child: process::Child) -> CompileFuture {
        CompileFuture(child)
    }
}

impl Future for CompileFuture {
    type Item = bool;
    type Error = ();
    fn poll(&mut self) -> Poll<bool, ()> {
        match tokio_threadpool::blocking(|| self.0.wait().unwrap().success()) {
            Ok(res) => Ok(res),
            Err(_) => Err(()),
        }
    }
}
