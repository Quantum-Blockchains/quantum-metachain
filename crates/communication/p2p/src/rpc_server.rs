use async_trait::async_trait;
use jsonrpc_http_server::jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_http_server::ServerBuilder;
use libp2p::mdns::Mdns;
use libp2p::Swarm;
use crate::error::P2PError;
use log::info;

pub struct DevRpcServer {
    pub rpc_server: ServerBuilder,
}

impl DevRpcServer {
    pub fn new(swarm: &mut Swarm<Mdns>) -> DevRpcServer {
        let mut handler: IoHandler = IoHandler::default();

        attach_handlers(&mut handler, swarm);

        DevRpcServer{
            rpc_server: ServerBuilder::new(handler).threads(3),
        }
    }
}

fn attach_handlers(handler: &mut IoHandler, swarm: &mut Swarm<Mdns>) {
    handler.add_method("say_hello", |_params: Params| async {
        Ok(Value::String(String::from("hello!")))
    })
}
