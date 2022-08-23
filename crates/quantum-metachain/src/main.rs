use qmc_p2p::config::P2PConfiguration;
use qmc_p2p::service::{DevP2PService, P2PService};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let p2p_config = P2PConfiguration {
        // TODO JEQB-79 read listen address from config
        listen_address: "/ip4/0.0.0.0/tcp/0".to_string(),
    };

    let p2p_service = DevP2PService::new(p2p_config);
    match p2p_service.start().await {
        Ok(_) => {}
        Err(err) => panic!("Cannot start p2p service: {:?}", err),
    };
    Ok(())
}
