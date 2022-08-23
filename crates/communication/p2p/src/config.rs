use config_file::{FromConfigFile, ConfigFileError};
use serde::Deserialize;

/// P2P service configuration
#[derive(Deserialize)]
pub struct P2PConfiguration {
    /// IP address to listen for incoming connections.
    pub listen_address: String,
}

pub fn new(path: &str) -> Result<P2PConfiguration, ConfigFileError> {
    println!("Trying to add config from path {}", path);

    return P2PConfiguration::from_config_file(std::path::Path::new(path));
}
