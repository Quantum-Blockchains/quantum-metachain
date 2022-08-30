use config_file::{ConfigFileError, FromConfigFile};
use log::info;
use serde::Deserialize;

/// P2P service configuration
#[derive(Deserialize)]
pub struct RpcConfiguration {
    /// IP address to listen for incoming connections.
    pub rpc_server_address: String,
}

/// Loads p2p config from a file.
pub fn new(path: &str) -> Result<RpcConfiguration, ConfigFileError> {
    info!("Trying to add config from path {}", path);

    return RpcConfiguration::from_config_file(std::path::Path::new(path));
}
