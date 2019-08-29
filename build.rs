use std::path::Path;

use protoc_grpcio;

const RPC_ROOT: &str = "src/rpc";
const RPC: &str = "rpc.proto";

fn main() {
    watch_changes();
    build_proto();
}

fn watch_changes() {
    println!("cargo:rerun-if-changed={}", RPC);
}

fn build_proto() {
    let proto_root = Path::new(RPC_ROOT);
    protoc_grpcio::compile_grpc_protos(&[RPC], &["."], &proto_root, None)
        .expect("Failed to compile proto");
}
