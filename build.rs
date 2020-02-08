use std::env;
use std::io;
use std::path::PathBuf;

use lazy_static::lazy_static;
use tonic_build;

#[cfg(any(feature = "seccomp", feature = "cap-ng"))]
use bindgen;

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
    println!("cargo:rerun-if-changed={}", "build.rs");

    build_proto()?;

    #[cfg(feature = "seccomp")]
    generate_libseccomp_binding()?;

    #[cfg(feature = "cap-ng")]
    generate_libcap_ng_binding()?;

    Ok(())
}

fn build_proto() -> io::Result<()> {
    const RPC_FILE: &str = "./rpc.proto";
    println!("cargo:rerun-if-changed={}", RPC_FILE);
    tonic_build::configure()
        .build_client(false)
        .build_server(true)
        .compile(&[RPC_FILE], &["."])?;
    Ok(())
}

#[cfg(feature = "seccomp")]
fn generate_libseccomp_binding() -> io::Result<()> {
    const SECCOMP_HEADER: &str = "/usr/include/seccomp.h";
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
    const CAPNG_HEADER: &str = "/usr/include/cap-ng.h";
    println!("cargo:rerun-if-changed={}", CAPNG_HEADER);
    println!("cargo:rustc-link-lib=dylib=cap-ng");
    bindgen::builder()
        .header_contents("cap-ng.h", "#include<cap-ng.h>")
        .generate()
        .expect("Failed to generate libcap-ng bindings")
        .write_to_file(OUT_DIR.join("libcapng.rs"))?;
    Ok(())
}
