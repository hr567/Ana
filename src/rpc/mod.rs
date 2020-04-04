mod rpc;

use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::process;

use async_trait::async_trait;
use futures::executor;
use futures::prelude::*;
use tokio::runtime::{self, Runtime};
use tokio::sync::mpsc;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use crate::judge;
use crate::workspace::Workspace;

pub struct RpcServer {
    runtime: Runtime,
}

impl RpcServer {
    pub fn new(max_threads: usize) -> RpcServer {
        let runtime = runtime::Builder::new()
            .core_threads(max_threads)
            .enable_all()
            .build()
            .expect("Failed to create a runtime");
        RpcServer { runtime }
    }

    pub fn start(self, address: IpAddr, port: u16) -> ! {
        let srv = rpc::ana_server::AnaServer::new(self);
        match executor::block_on(
            Server::builder()
                .add_service(srv)
                .serve(SocketAddr::new(address, port)),
        ) {
            Ok(()) => process::exit(0),
            Err(e) => panic!("Serve stopped. {}", e),
        }
    }
}

#[async_trait]
impl rpc::ana_server::Ana for RpcServer {
    type JudgeWorkspaceStream =
        Pin<Box<dyn Stream<Item = Result<rpc::Report, Status>> + Unpin + Send + Sync + 'static>>;
    async fn judge_workspace(
        &self,
        request: Request<rpc::Workspace>,
    ) -> Result<Response<Self::JudgeWorkspaceStream>, Status> {
        let (tx, rx) = mpsc::unbounded_channel();
        let workspace_path = match &request.get_ref().path {
            Some(path) => path,
            None => return Err(Status::data_loss("path of workspace is not exist")),
        };
        let workspace = match Workspace::from_path(workspace_path) {
            Ok(workspace) => workspace,
            Err(e) => {
                return Err(Status::unavailable(format!(
                    "Failed to get workspace. {}",
                    e
                )))
            }
        };
        self.runtime.spawn(async move {
            match judge::judge(workspace, tx.clone()).await {
                Ok(()) => {}
                Err(e) => {
                    let _ = tx.send(judge::Report {
                        result: judge::ResultType::SystemError,
                        usage: None,
                        message: format!("Failed to judge task. {}", e),
                    });
                }
            }
        });

        let res: Box<
            dyn Stream<Item = Result<rpc::Report, Status>> + Send + Sync + Unpin + 'static,
        > =
            Box::new(rx.map(|report| -> Result<rpc::Report, tonic::Status> {
                Ok(rpc::Report::from(report))
            }));
        Ok(Response::new(Pin::new(res)))
    }
}
