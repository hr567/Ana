use std::path::Path;

use protoc_grpcio;

fn main() {
    build_proto();
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
