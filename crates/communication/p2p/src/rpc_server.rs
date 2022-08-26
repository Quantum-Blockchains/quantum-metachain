use async_trait::async_trait;
use jsonrpc_http_server::jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_http_server::ServerBuilder;
use crate::error::P2PError;
use log::info;

pub struct DevRpcServer {
    pub rpc_server: ServerBuilder,
}

impl DevRpcServer {
    pub fn new() -> DevRpcServer {
        let mut handler = IoHandler::default();

        // Implement method to attach handlers separately
        handler.add_method("say_hello", |_params: Params| async {
            Ok(Value::String("hello".to_owned()))
        });

        DevRpcServer{
            rpc_server: ServerBuilder::new(handler).threads(3),
        }
    }
}
