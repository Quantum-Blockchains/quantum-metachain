use qmc_p2p::config;
use qmc_p2p::service::{DevP2PService, P2PService};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let p2p_config = match config::new("./config/p2p/config.toml") {
        Ok(c) => c,
        Err(err) => panic!("Couldn't load config file: {:?}", err),
    };

    // basic printing until logger is introduced
    println!(
        "Found config file:\n- listen_address: {}\n--------",
        p2p_config.listen_address
    );

    let p2p_service = DevP2PService::new(p2p_config);
    match p2p_service.start().await {
        Ok(_) => {}
        Err(err) => panic!("Cannot start p2p service: {:?}", err),
    };
    Ok(())
}
