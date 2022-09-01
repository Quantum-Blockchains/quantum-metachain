use crate::error::RPCError;
use jsonrpc_http_server::jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_http_server::ServerBuilder;

pub struct DevRpcServer {
    pub rpc_server: ServerBuilder,
}

pub trait RpcServer {
    fn start(self) -> Result<(), RPCError>;
}

impl Default for DevRpcServer {
    fn default() -> Self {
        Self::new()
    }
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

fn attach_handlers(handler: &mut IoHandler) {
    handler.add_method("say_hello", |_params: Params| async {
        Ok(Value::String(String::from("hello!")))
    });
    handler.add_method("say_hello_to_peers", |_params: Params| async {
        Ok(Value::String(String::from("hello!")))
    });
}
