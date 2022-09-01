use async_trait::async_trait;

use crate::config::P2PConfiguration;
use crate::error::P2PError;

use libp2p::futures::StreamExt;
use libp2p::mdns::{Mdns, MdnsEvent};
use libp2p::swarm::SwarmEvent;
use libp2p::Swarm;

use log::info;

/// P2P service
#[async_trait]
pub trait P2PService {
    /// Starts p2p network
    async fn start(self) -> Result<(), P2PError>;
}

/// Dev p2p service
///
/// For development purposes only
pub struct DevP2PService {
    config: P2PConfiguration,
    swarm: Swarm<Mdns>,
}

impl DevP2PService {
    pub fn new(config: P2PConfiguration, swarm: Swarm<Mdns>) -> DevP2PService {
        DevP2PService { config, swarm }
    }
}

#[async_trait]
impl P2PService for DevP2PService {
    async fn start(mut self) -> Result<(), P2PError> {
        self.swarm
            .listen_on(match self.config.listen_address.parse() {
                Ok(m) => m,
                Err(_err) => return Err(P2PError::ParsingAddressError(self.config.listen_address)),
            })?;

        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Listening on local address {:?}", address)
                }
                SwarmEvent::Behaviour(MdnsEvent::Discovered(peers)) => {
                    for (peer, addr) in peers {
                        info!("discovered {} {}", peer, addr);
                    }
                }
                SwarmEvent::Behaviour(MdnsEvent::Expired(expired)) => {
                    for (peer, addr) in expired {
                        info!("expired {} {}", peer, addr);
                    }
                }
                _ => {}
            }
        }
    }
}
