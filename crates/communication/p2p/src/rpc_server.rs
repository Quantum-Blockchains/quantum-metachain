use async_trait::async_trait;
use jsonrpc_http_server::jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_http_server::ServerBuilder;
use crate::error::P2PError;
use log::info;

#[async_trait]
pub trait RpcServer {
    async fn start(self) -> Result<(), P2PError>;
}

pub struct DevRpcServer {
    rpc_server: ServerBuilder,
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

#[async_trait]
impl RpcServer for DevRpcServer {
    async fn start(self) -> Result<(), P2PError> {
        let result = match self.rpc_server.start_http(&"127.0.0.1:3030".parse().unwrap()) {
            Ok(_) => info!("Started RPC server"),
            Err(err) => return Err(P2PError::IOError(err))
        };

        Ok(())
    }
}
