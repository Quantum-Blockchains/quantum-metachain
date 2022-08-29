use jsonrpc_http_server::jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_http_server::ServerBuilder;
use libp2p::mdns::Mdns;
use libp2p::Swarm;
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
    handler.add_method("say_hello_to_peers", |_params: Params| async {
        // creates errors, remove contents of this method to fix compilation errors
        let peers = swarm.listeners();
        for peer in peers {
            // ping all of our peers here, something like swarm.dial(peer.parse())?
        }
        Ok(Value::String(String::from("Said hello to many peers!")))
    });

    handler.add_method("say_hello", |_params: Params| async {
        Ok(Value::String(String::from("hello!")))
    });
}
