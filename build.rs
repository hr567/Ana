extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=zmq");

    bindgen::Builder::default()
        .header("externals/zmq/zmq.h")
        .generate()
        .expect("Cannot generate header file")
        .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("zmq.rs"))
        .expect("Cannot write to header file");
}
