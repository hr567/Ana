mod ana_rpc;

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::process;

use async_trait::async_trait;
use futures::executor;
use futures::prelude::*;
use lazy_static::lazy_static;
use tokio::runtime::{self, Runtime};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tokio::sync::RwLock;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use crate::judge;
use crate::workspace::Workspace;
use ana_rpc as rpc;

lazy_static! {
    static ref REGISTER: Register = Register::new();
}

struct Register {
    register: RwLock<HashMap<String, RwLock<UnboundedReceiver<rpc::Report>>>>,
}

impl Register {
    fn new() -> Register {
        Register {
            register: RwLock::new(HashMap::new()),
        }
    }

    async fn register<T: Stream<Item = rpc::Report> + Send + 'static>(
        &self,
        id: String,
        reports: T,
    ) {
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(reports.for_each(move |report| {
            let _ = tx.send(report);
            future::ready(())
        }));
        let mut register = self.register.write().await;
        if register.contains_key(&id) {
            log::warn!("the task ID {} is exist. The new one is ignored.", &id);
            return;
        }
        register.insert(id, RwLock::new(rx));
    }

    async fn get_report(&self, id: &str) -> Option<rpc::Report> {
        let res = {
            let register = self.register.read().await;
            let mut rx = register.get(id)?.write().await;
            rx.recv().await
        };
        if res.is_none() {
            let mut register = self.register.write().await;
            register.remove(id);
        }
        res
    }
}

pub struct RpcServer {
    runtime: Runtime,
}

impl RpcServer {
    pub fn new(max_threads: usize) -> RpcServer {
        let runtime = runtime::Builder::new()
            .threaded_scheduler()
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
    async fn judge_workspace(
        &self,
        request: Request<rpc::Workspace>,
    ) -> Result<Response<()>, Status> {
        let request = request.into_inner();
        let id = match request.id {
            Some(id) => id,
            None => return Err(Status::data_loss("id of task is not exist")),
        };
        let path = match request.path {
            Some(path) => path,
            None => return Err(Status::data_loss("path of workspace is not exist")),
        };
        let workspace = match Workspace::from_path(path) {
            Ok(workspace) => workspace,
            Err(e) => {
                return Err(Status::unavailable(format!(
                    "failed to generate workspace at {}",
                    e
                )))
            }
        };
        let (tx, rx) = mpsc::unbounded_channel();
        self.runtime.spawn(async move {
            if let Err(e) = judge::judge(workspace, tx.clone()).await {
                let _ = tx.send(judge::Report {
                    result: judge::ResultType::SystemError,
                    usage: None,
                    message: format!("Failed to judge task. {}", e),
                });
            }
        });
        REGISTER.register(id, rx.map(rpc::Report::from)).await;
        Ok(Response::new(()))
    }

    async fn get_report(
        &self,
        request: Request<rpc::Request>,
    ) -> Result<Response<rpc::Report>, Status> {
        let id = match request.into_inner().id {
            Some(id) => id,
            None => return Err(Status::data_loss("id of task is missing")),
        };
        match REGISTER.get_report(&id).await {
            Some(report) => Ok(Response::new(report)),
            None => Err(Status::out_of_range(
                "the task is finished or does not exist",
            )),
        }
    }
}
