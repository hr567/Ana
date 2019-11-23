use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use bincode;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json;

#[cfg(any(feature = "seccomp", feature = "cap-ng"))]
use bindgen;

const SECCOMP_HEADER: &str = "/usr/include/seccomp.h";
const CAPNG_HEADER: &str = "/usr/include/cap-ng.h";

lazy_static! {
    static ref OUT_DIR: PathBuf = {
        let out_dir = env::var("OUT_DIR").unwrap();
        PathBuf::from(out_dir)
    };
    static ref ROOT_DIR: PathBuf = {
        let root_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        PathBuf::from(root_dir)
    };
}

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed={}", "Cargo.toml");

    #[cfg(feature = "rpc")]
    build_proto()?;

    #[cfg(feature = "seccomp")]
    generate_libseccomp_binding()?;

    #[cfg(feature = "cap-ng")]
    generate_libcap_ng_binding()?;

    Ok(())
}

#[cfg(feature = "rpc")]
fn build_proto() -> io::Result<()> {
    use tonic_build;
    const RPC_FILE: &str = "./rpc.proto";
    println!("cargo:rerun-if-changed={}", RPC_FILE);
    tonic_build::compile_protos(RPC_FILE)?;
    Ok(())
}

#[cfg(feature = "seccomp")]
fn generate_libseccomp_binding() -> io::Result<()> {
    println!("cargo:rerun-if-changed={}", SECCOMP_HEADER);
    println!("cargo:rustc-link-lib=dylib=seccomp");
    bindgen::builder()
        .header_contents("seccomp.h", "#include<seccomp.h>")
        .generate()
        .expect("Failed to generate libseccomp bindings")
        .write_to_file(OUT_DIR.join("libseccomp.rs"))?;
    Ok(())
}

#[cfg(feature = "cap-ng")]
fn generate_libcap_ng_binding() -> io::Result<()> {
    println!("cargo:rerun-if-changed={}", CAPNG_HEADER);
    println!("cargo:rustc-link-lib=dylib=cap-ng");
    bindgen::builder()
        .header_contents("cap-ng.h", "#include<cap-ng.h>")
        .generate()
        .expect("Failed to generate libcap-ng bindings")
        .write_to_file(OUT_DIR.join("libcapng.rs"))?;
    Ok(())
}
