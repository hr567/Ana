use bindgen;

fn main() {
    println!("cargo:rustc-link-lib=dylib=seccomp");
    let seccomp_bindings = bindgen::builder()
        .header_contents("seccomp_wrapper.h", "#include<seccomp.h>")
        // .whitelist_function("seccomp_init")
        // .whitelist_function("seccomp_rule_add")
        // .whitelist_function("seccomp_load")
        // .whitelist_function("seccomp_release")
        // .whitelist_function("seccomp_syscall_resolve_name")
        .generate()
        .expect("Failed to generate seccomp bindings");

    seccomp_bindings
        .write_to_file("src/runner/seccomp/warpper.rs")
        .expect("Failed to write to seccomp ffi file");
}
