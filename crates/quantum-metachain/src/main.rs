use log::info;
use qmc_p2p::config;
use qmc_p2p::service::{DevP2PService, P2PService};
use qmc_rpc::rpc_server::{DevRpcServer, RpcServer};
mod logger;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    logger::init();
    let p2p_config = match config::new("./config/p2p/config.toml") {
        Ok(c) => c,
        Err(err) => panic!("Couldn't load config file: {:?}", err),
    };

    info!(
        "Found config file:\n- listen_address: {}\n",
        p2p_config.listen_address,
    );

    let rpc_server = DevRpcServer::new();
    let _result = rpc_server.rpc_server.start_http(&"127.0.0.1:3030".parse().unwrap()).unwrap();
    info!("RPC server starded, listening on: ");

    let p2p_service = DevP2PService::new(p2p_config);
    match p2p_service.start().await {
        Ok(_) => {}
        Err(err) => panic!("Cannot start p2p service: {:?}", err),
    };
    Ok(())
}
