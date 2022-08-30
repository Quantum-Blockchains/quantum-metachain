use std::fmt::Error;
use jsonrpc_http_server::jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_http_server::ServerBuilder;
use libp2p::mdns::Mdns;
use libp2p::Swarm;
use crate::error::RPCError;
use async_trait::async_trait;

pub struct DevRpcServer {
    pub rpc_server: ServerBuilder,
}

#[async_trait]
pub trait RpcServer {
    async fn start(self) -> Result<(), RPCError>;
}

impl DevRpcServer {
    pub fn new() -> DevRpcServer {
        let mut handler: IoHandler = IoHandler::default();

        attach_handlers(&mut handler);

        DevRpcServer {
            rpc_server: ServerBuilder::new(handler).threads(3),
        }
    }
}

#[async_trait]
impl RpcServer for DevRpcServer {
    async fn start(self) -> Result<(), RPCError> {
        println!("Trying to start::::::::::::::::::");
        let _result = self.rpc_server.start_http(&"127.0.0.1:3030".parse().unwrap());

        Ok(())
    }
}

fn attach_handlers(handler: &mut IoHandler) {
    handler.add_method("say_hello", |_params: Params| async {
        Ok(Value::String(String::from("hello!")))
    });
}
