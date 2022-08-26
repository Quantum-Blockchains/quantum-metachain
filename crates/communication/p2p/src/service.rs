use async_trait::async_trait;

use crate::config::P2PConfiguration;
use crate::error::P2PError;
use crate::rpc_server::DevRpcServer;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use libp2p::futures::StreamExt;
use libp2p::identity::Keypair;
use libp2p::mdns::{Mdns, MdnsConfig, MdnsEvent};
use libp2p::swarm::SwarmEvent;
use libp2p::{PeerId, Swarm};

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
    id_keys: Keypair,
    rpc_server: DevRpcServer,
}

impl DevP2PService {
    pub fn new(config: P2PConfiguration, rpc_server: DevRpcServer) -> DevP2PService {
        let id_keys = Keypair::generate_ed25519();
        DevP2PService { config, id_keys, rpc_server }
    }
}

#[async_trait]
impl P2PService for DevP2PService {
    async fn start(self) -> Result<(), P2PError> {
        let transport = libp2p::development_transport(self.id_keys.clone()).await?;
        let behaviour = Mdns::new(MdnsConfig::default()).await?;
        let peer_id = PeerId::from(self.id_keys.public());

        let str_addr = self.config.rpc_server_address.as_str();
        let server = self.rpc_server.rpc_server.start_http(
            &str_addr.parse().unwrap()).unwrap();
        info!("RPC server listening at address {}", self.config.rpc_server_address);

        let mut swarm = Swarm::new(transport, behaviour, peer_id);
        swarm.listen_on(match self.config.listen_address.parse() {
            Ok(m) => m,
            Err(_err) => return Err(P2PError::ParsingAddressError(self.config.listen_address)),
        })?;
        loop {
            match swarm.select_next_some().await {
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
