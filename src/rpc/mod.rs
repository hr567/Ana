mod rpc;

use std::pin::Pin;

use futures::Stream;
use rpc::server;
use rpc::{Report, Task};
use tonic;
use tonic::{Request, Response, Status};

use crate::workspace::Workspace;

#[derive(Default)]
struct RpcServer;

#[tonic::async_trait]
impl server::Ana for RpcServer {
    type JudgeStream = Pin<Box<dyn Stream<Item = Result<Report, Status>> + Send + Sync + 'static>>;

    async fn judge(&self, _request: Request<Task>) -> Result<Response<Self::JudgeStream>, Status> {
        let workspace = Workspace::new().await?;
        unimplemented!()
    }
}
