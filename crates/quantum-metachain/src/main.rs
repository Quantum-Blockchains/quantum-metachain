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
    match rpc_server.start().await {
        Ok(_) => {}
        Err(err) => panic!("Cannot start RPC server: {:?}", err)
    }
    info!("Started");

    let p2p_service = DevP2PService::new(p2p_config);
    match p2p_service.start().await {
        Ok(_) => {}
        Err(err) => panic!("Cannot start p2p service: {:?}", err),
    };
    Ok(())
}
