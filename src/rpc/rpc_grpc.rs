// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

const METHOD_ANA_JUDGE: ::grpcio::Method<super::rpc::Task, super::rpc::Report> = ::grpcio::Method {
    ty: ::grpcio::MethodType::ServerStreaming,
    name: "/Ana/judge",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct AnaClient {
    client: ::grpcio::Client,
}

impl AnaClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        AnaClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn judge_opt(&self, req: &super::rpc::Task, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientSStreamReceiver<super::rpc::Report>> {
        self.client.server_streaming(&METHOD_ANA_JUDGE, req, opt)
    }

    pub fn judge(&self, req: &super::rpc::Task) -> ::grpcio::Result<::grpcio::ClientSStreamReceiver<super::rpc::Report>> {
        self.judge_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Item = (), Error = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait Ana {
    fn judge(&mut self, ctx: ::grpcio::RpcContext, req: super::rpc::Task, sink: ::grpcio::ServerStreamingSink<super::rpc::Report>);
}

pub fn create_ana<S: Ana + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s;
    builder = builder.add_server_streaming_handler(&METHOD_ANA_JUDGE, move |ctx, req, resp| {
        instance.judge(ctx, req, resp)
    });
    builder.build()
}
