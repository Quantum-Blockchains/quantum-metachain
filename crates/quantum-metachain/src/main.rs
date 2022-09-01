use libp2p::identity::Keypair;
use libp2p::mdns::{Mdns, MdnsConfig};
use libp2p::{PeerId, Swarm};
use log::info;
use qmc_p2p::service::{DevP2PService, P2PService};
use qmc_rpc::rpc_server::DevRpcServer;
mod logger;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    logger::init();
    let p2p_config = match qmc_p2p::config::new("./config/p2p/config.toml") {
        Ok(c) => c,
        Err(err) => panic!("Couldn't load config file: {:?}", err),
    };
    info!(
        "Found config file:\n- listen_address: {}\n",
        p2p_config.listen_address,
    );

    let rpc_config = match qmc_rpc::config::new("./config/rpc/config.toml") {
        Ok(c) => c,
        Err(err) => panic!("Couldn't load config file: {:?}", err),
    };
    info!(
        "Found config file:\n- rpc_server_address: {}\n",
        rpc_config.rpc_server_address,
    );

    let id_keys = Keypair::generate_ed25519();
    let transport = libp2p::development_transport(id_keys.clone()).await?;
    let behaviour = Mdns::new(MdnsConfig::default()).await?;
    let peer_id = PeerId::from(id_keys.public());
    let swarm = Swarm::new(transport, behaviour, peer_id);

    let str_addr = rpc_config.rpc_server_address.as_str();

    let rpc_server = DevRpcServer::new();
    let _result = rpc_server
        .rpc_server
        .start_http(&str_addr.parse().unwrap())
        .unwrap();
    info!("RPC server starded, listening on: ");

    let p2p_service = DevP2PService::new(p2p_config, swarm);
    match p2p_service.start().await {
        Ok(_) => {}
        Err(err) => panic!("Cannot start p2p service: {:?}", err),
    };
    Ok(())
}
