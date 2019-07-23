use std::path::Path;

use protoc_grpcio;

fn main() {
    watch_changes();
    build_proto();
}

fn watch_changes() {
    println!("cargo:rerun-if-changed={}", "/src/rpc/rpc.proto");
}

fn build_proto() {
    let proto_root = Path::new("src/rpc");
    protoc_grpcio::compile_grpc_protos(
        &[proto_root.join("rpc.proto")],
        &[&proto_root],
        &proto_root,
        None,
    )
    .expect("Failed to compile proto");
}
