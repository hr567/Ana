pub mod builder;
pub mod comparer;
pub mod judge;
pub mod language;
pub mod process;
pub mod rpc;
pub mod runner;
pub mod workspace;

use std::net::IpAddr;

pub fn start_rpc_server(address: IpAddr, port: u16, threads: usize) {
    rpc::RpcServer::new(threads).start(address, port);
}

#[cfg(test)]
mod tests;
